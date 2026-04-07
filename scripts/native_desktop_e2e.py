from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

from PIL import ImageGrab

try:
    from pywinauto import Desktop, keyboard, mouse
except ImportError as error:  # pragma: no cover - exercised only when dependency is missing
    print(
        "pywinauto is required for native desktop automation. Install it with "
        "`python -m pip install pywinauto`.",
        file=sys.stderr,
    )
    raise SystemExit(1) from error


ROOT = Path(__file__).resolve().parents[1]
ARTIFACTS_ROOT = ROOT / "artifacts" / "saves"
RESULTS_ROOT = ROOT / "test-results" / "native-desktop-e2e"
WINDOW_TITLE = "FutureAero"
APP_URL = "http://127.0.0.1:1420"


class NativeDesktopError(RuntimeError):
    pass


def wait_for_http(url: str, timeout_seconds: int) -> None:
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None
    while time.time() < deadline:
      try:
        with urllib.request.urlopen(url, timeout=2) as response:
          if response.status == 200:
            return
      except Exception as error:  # pragma: no cover - timing dependent
        last_error = error
      time.sleep(1)
    raise NativeDesktopError(f"Timed out waiting for {url}: {last_error}")


def find_window(timeout_seconds: int):
    deadline = time.time() + timeout_seconds
    last_error: Exception | None = None
    while time.time() < deadline:
        try:
            window = Desktop(backend="win32").window(title=WINDOW_TITLE, class_name="Tauri Window")
            window.wait("exists enabled visible ready", timeout=2)
            return window
        except Exception as error:  # pragma: no cover - timing dependent
            last_error = error
            time.sleep(1)

    raise NativeDesktopError(f"Timed out waiting for desktop window `{WINDOW_TITLE}`: {last_error}")


def launch_dev_shell() -> int:
    npm_executable = shutil.which("npm.cmd") or shutil.which("npm")
    if not npm_executable:
        raise NativeDesktopError("Unable to locate npm or npm.cmd in PATH.")
    command = (
        f"$npm = '{npm_executable}'; "
        f"$p = Start-Process -FilePath $npm "
        f"-ArgumentList 'run','dev' "
        f"-WorkingDirectory '{ROOT}' "
        f"-PassThru; "
        "$p.Id"
    )
    result = subprocess.run(
        ["powershell", "-NoProfile", "-Command", command],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    if not lines:
        raise NativeDesktopError(
            f"Unable to retrieve the dev shell PID from PowerShell. stdout=`{result.stdout}` stderr=`{result.stderr}`"
        )
    pid = lines[-1]
    return int(pid)


def terminate_process_tree(process_id: int | None) -> None:
    if process_id is None:
        return

    subprocess.run(
        ["taskkill", "/PID", str(process_id), "/T", "/F"],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def terminate_existing_futureaero_shells() -> None:
    subprocess.run(
        ["taskkill", "/IM", "futureaero-desktop-shell.exe", "/T", "/F"],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    deadline = time.time() + 10
    while time.time() < deadline:
        try:
            Desktop(backend="win32").window(title=WINDOW_TITLE, class_name="Tauri Window").wait_not(
                "exists",
                timeout=1,
            )
            return
        except Exception:
            time.sleep(0.5)


def terminate_processes_on_port(port: int) -> None:
    netstat = subprocess.run(
        ["netstat", "-ano", "-p", "tcp"],
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    pids: set[str] = set()
    for line in netstat.stdout.splitlines():
        columns = line.split()
        if len(columns) < 5:
            continue
        local_address = columns[1]
        state = columns[3]
        pid = columns[4]
        if local_address.endswith(f":{port}") and state.upper() == "LISTENING":
            pids.add(pid)

    for pid in sorted(pids):
        subprocess.run(
            ["taskkill", "/PID", pid, "/T", "/F"],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )


def snapshot_hash(window, image_name: str) -> str:
    image_path = RESULTS_ROOT / image_name
    rect = window.rectangle()
    image = ImageGrab.grab(bbox=(rect.left, rect.top, rect.right, rect.bottom))
    image.save(image_path)
    return hashlib.sha256(image.tobytes()).hexdigest()


def looks_like_blank_shell(window) -> bool:
    rect = window.rectangle()
    image = ImageGrab.grab(bbox=(rect.left, rect.top, rect.right, rect.bottom))
    width, height = image.size
    crop = image.crop((0, 30, width, height))
    bright_samples = 0
    dark_samples = 0
    total_samples = 0

    for y in range(0, crop.size[1], 40):
        for x in range(0, crop.size[0], 40):
            red, green, blue = crop.getpixel((x, y))[:3]
            total_samples += 1
            if red >= 245 and green >= 245 and blue >= 245:
                bright_samples += 1
            if red <= 24 and green <= 24 and blue <= 24:
                dark_samples += 1

    if total_samples == 0:
        return True

    bright_ratio = bright_samples / total_samples
    dark_ratio = dark_samples / total_samples
    return bright_ratio >= 0.92 or dark_ratio >= 0.9


def wait_for_rendered_shell(window, timeout_seconds: int) -> None:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        if not looks_like_blank_shell(window):
            return
        time.sleep(1)

    raise NativeDesktopError("Timed out waiting for the rendered FutureAero shell content.")


def focus_workspace(window) -> None:
    window.restore()
    window.set_focus()
    rect = window.rectangle()
    workspace_focus_point = (
        rect.left + int(rect.width() * 0.52),
        rect.top + int(rect.height() * 0.30),
    )
    mouse.click(
        button="left",
        coords=workspace_focus_point,
    )
    time.sleep(1.5)


def collect_saved_projects() -> set[Path]:
    if not ARTIFACTS_ROOT.exists():
        return set()
    return {path.resolve() for path in ARTIFACTS_ROOT.glob("*.faero")}


def wait_for_new_saved_project(before: set[Path], timeout_seconds: int, description: str) -> Path:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        current = collect_saved_projects()
        difference = sorted(current - before)
        if difference:
            return difference[-1]
        time.sleep(1)

    raise NativeDesktopError(f"Timed out waiting for saved project after {description}")


def read_project_name(project_directory: Path) -> str:
    project_file = project_directory / "project.yaml"
    if not project_file.exists():
        raise NativeDesktopError(f"Missing project.yaml in {project_directory}")

    for line in project_file.read_text(encoding="utf-8").splitlines():
        if line.startswith("name: "):
            return line.split(": ", 1)[1].strip()

    raise NativeDesktopError(f"Unable to read project name in {project_file}")


def send_shortcut(window, keys: str, settle_seconds: float = 1.0) -> None:
    focus_workspace(window)
    keyboard.send_keys(keys, pause=0.05, with_spaces=True)
    time.sleep(settle_seconds)


def run_native_suite(window) -> list[dict[str, str]]:
    RESULTS_ROOT.mkdir(parents=True, exist_ok=True)
    summaries: list[dict[str, str]] = []
    time.sleep(12)
    wait_for_rendered_shell(window, 60)
    focus_workspace(window)
    snapshot_hash(window, "baseline.png")

    before_initial_save = collect_saved_projects()
    send_shortcut(window, "^s")
    initial_save = wait_for_new_saved_project(before_initial_save, 20, "Ctrl+S")
    initial_name = read_project_name(initial_save)
    if initial_name != "Pick And Place Demo":
        raise NativeDesktopError(
            f"Unexpected project saved by Ctrl+S: expected `Pick And Place Demo`, got `{initial_name}`"
        )
    summaries.append(
        {
            "name": "ctrl_s_saves_current_fixture",
            "result": "passed",
            "artifact": str(initial_save.relative_to(ROOT)),
        }
    )

    send_shortcut(window, "^+n")
    before_session_save = collect_saved_projects()
    send_shortcut(window, "^s")
    session_save = wait_for_new_saved_project(before_session_save, 20, "Ctrl+Shift+N then Ctrl+S")
    session_name = read_project_name(session_save)
    if session_name != "FutureAero Session":
        raise NativeDesktopError(
            f"Unexpected project saved after Ctrl+Shift+N: expected `FutureAero Session`, got `{session_name}`"
        )
    summaries.append(
        {
            "name": "ctrl_shift_n_then_ctrl_s_saves_new_session",
            "result": "passed",
            "artifact": str(session_save.relative_to(ROOT)),
        }
    )

    before_save_all = collect_saved_projects()
    send_shortcut(window, "^+s")
    save_all = wait_for_new_saved_project(before_save_all, 20, "Ctrl+Shift+S")
    save_all_name = read_project_name(save_all)
    if save_all_name != "FutureAero Session":
        raise NativeDesktopError(
            f"Unexpected project saved by Ctrl+Shift+S: expected `FutureAero Session`, got `{save_all_name}`"
        )
    summaries.append(
        {
            "name": "ctrl_shift_s_saves_all_current_session",
            "result": "passed",
            "artifact": str(save_all.relative_to(ROOT)),
        }
    )

    before_f4_hash = snapshot_hash(window, "before-f4.png")
    send_shortcut(window, "{F4}")
    after_f4_hash = snapshot_hash(window, "after-f4.png")
    if before_f4_hash == after_f4_hash:
        raise NativeDesktopError("F4 did not produce a visible desktop change in the captured window.")
    send_shortcut(window, "%{ENTER}")
    restored_hash = snapshot_hash(window, "after-alt-enter.png")
    if restored_hash == after_f4_hash:
        raise NativeDesktopError("Alt+Enter did not restore a visible desktop change after F4.")
    summaries.append(
        {
            "name": "f4_and_alt_enter_toggle_properties_panel",
            "result": "passed",
            "artifact": str((RESULTS_ROOT / "after-alt-enter.png").relative_to(ROOT)),
        }
    )

    return summaries


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run native Windows desktop automation against the real FutureAero Tauri shell."
    )
    parser.add_argument(
        "--reuse-existing",
        action="store_true",
        help="Attach to an already running FutureAero window instead of launching `npm run dev`.",
    )
    parser.add_argument(
        "--window-timeout",
        type=int,
        default=180,
        help="Timeout, in seconds, used to wait for the dev server and the desktop window.",
    )
    return parser.parse_args()


def main() -> int:
    if os.name != "nt":
        print("Native desktop automation is only supported on Windows.", file=sys.stderr)
        return 1

    if RESULTS_ROOT.exists():
        shutil.rmtree(RESULTS_ROOT)
    RESULTS_ROOT.mkdir(parents=True, exist_ok=True)

    args = parse_args()
    launched_process_id: int | None = None

    try:
        if not args.reuse_existing:
            terminate_existing_futureaero_shells()
            terminate_processes_on_port(1420)
            launched_process_id = launch_dev_shell()
            wait_for_http(APP_URL, args.window_timeout)

        window = find_window(args.window_timeout)
        summaries = run_native_suite(window)

        summary_path = RESULTS_ROOT / "summary.json"
        summary_path.write_text(json.dumps(summaries, indent=2), encoding="utf-8")
        print(json.dumps({"status": "passed", "checks": summaries}, indent=2))
        return 0
    except NativeDesktopError as error:
        print(str(error), file=sys.stderr)
        return 1
    finally:
        terminate_process_tree(launched_process_id)


if __name__ == "__main__":
    raise SystemExit(main())
