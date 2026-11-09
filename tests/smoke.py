#!/usr/bin/env python3
"""Drive the actual Linux browser against a loopback HTTP fixture."""

import contextlib
import http.server
import json
import os
from pathlib import Path
import select
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time


PAYLOAD = b"Crest Browser smoke test\n"


class Fixture(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        if self.path == "/attachment":
            body = PAYLOAD
            self.send_header("Content-Type", "application/octet-stream")
            self.send_header("Content-Disposition", 'attachment; filename="crest-smoke.txt"')
        else:
            name = {"/a": "A", "/b": "B", "/popup": "popup"}.get(self.path, "other")
            body = f"""<!doctype html><meta charset="utf-8">
                <title>Fixture booting</title><h1>Crest fixture {name}</h1>
                <script>
                let count = 0;
                const name = {json.dumps(name)};
                const label = "Crest fixture " + name;
                document.title = label;
                window.addEventListener("keydown", (event) => {{
                    if (event.key === "j") document.title = label + " count " + (++count);
                    if (event.key === "d") location.href = "/attachment";
                    if (event.key === "n") window.open("/popup", "_blank");
                    if (event.key === "p") navigator.geolocation.getCurrentPosition(
                        () => document.title = "Geolocation allowed",
                        (error) => document.title = "Geolocation error " + error.code);
                }});
                </script>""".encode()
            self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass


def until(description, predicate, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = predicate()
        if result:
            return result
        time.sleep(0.06)
    raise AssertionError(f"Timed out waiting for {description}")


@contextlib.contextmanager
def private_display(env):
    with subprocess.Popen(
        ["Xvfb", "-displayfd", "1", "-screen", "0", "1280x900x24", "-nolisten", "tcp"],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    ) as server:
        try:
            if not select.select([server.stdout], [], [], 8)[0]:
                raise RuntimeError("Xvfb failed to start")
            display = server.stdout.readline().strip()
            if not display.isdigit():
                raise RuntimeError("Xvfb did not provide a display")
            yield {**env, "DISPLAY": f":{display}"}
        finally:
            server.terminate()


class Desktop:
    def __init__(self, env):
        self.env = env

    def xdo(self, *args, check=True):
        return subprocess.run(
            ["xdotool", *map(str, args)],
            env=self.env,
            check=check,
            capture_output=True,
            text=True,
            timeout=5,
        ).stdout.strip()

    def window(self, name):
        return until(
            name,
            lambda: next(iter(self.xdo("search", "--onlyvisible", "--name", name, check=False).splitlines()), ""),
        )

    def title(self, window, label):
        until(label, lambda: self.xdo("getwindowname", window) == f"Crest Browser — {label}")

    def key(self, *keys):
        self.xdo("key", "--clearmodifiers", *keys)

    def focus(self, window):
        self.xdo("windowfocus", "--sync", window)

    def navigate(self, url):
        self.key("ctrl+l")
        self.xdo("type", "--clearmodifiers", url)
        self.key("Return")

    def click(self, x, y):
        self.xdo("mousemove", x, y, "click", 1)

    def close(self, window, process):
        def close_if_visible():
            if process.poll() is not None or not self.xdo("getwindowname", window, check=False):
                return True
            # DevTools may still be raising its window after it first maps.
            self.xdo("windowraise", window, check=False)
            self.xdo("windowfocus", "--sync", window, check=False)
            self.click(1000, 500)
            self.key("ctrl+shift+w")
            return False

        until("main window to close", close_if_visible)
        assert process.wait(timeout=10) == 0


@contextlib.contextmanager
def launch(binary, env, directory):
    log = directory / "browser.log"
    with log.open("wb") as output:
        process = subprocess.Popen(
            ["dbus-run-session", "--", str(binary), "--profile", str(directory / "profile")],
            env=env,
            stdout=output,
            stderr=output,
            start_new_session=True,
        )
        try:
            yield process
        except Exception:
            print(log.read_text(errors="replace")[-5000:], file=sys.stderr)
            raise
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)


def profile(directory):
    try:
        return json.loads((directory / "profile" / "profile.json").read_text())
    except FileNotFoundError:
        return {}


def sandboxed_process(engine_profile):
    flag = os.fsencode(f"--user-data-dir={engine_profile}")
    for process in Path("/proc").glob("[0-9]*"):
        try:
            args = (process / "cmdline").read_bytes().split(b"\0")
            if flag not in args:
                continue
            status = dict(line.split(":", 1) for line in (process / "status").read_text().splitlines())
            if (
                status.get("Seccomp", "").strip() == "2"
                and status.get("NoNewPrivs", "").strip() == "1"
                and len(status.get("NSpid", "").split()) > 1
            ):
                return True
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            continue
    return False


def smoke(binary, directory, env, origin):
    desktop = Desktop(env)
    with launch(binary, env, directory) as process:
        main = desktop.window("^Crest Browser$")
        desktop.focus(main)
        desktop.navigate(origin + "/a")
        desktop.title(main, "Crest fixture A")
        desktop.key("alt+Left")
        until("back to new tab", lambda: desktop.xdo("getwindowname", main) == "Crest Browser")
        desktop.key("alt+Right")
        desktop.title(main, "Crest fixture A")
        desktop.key("j", "ctrl+d")
        desktop.title(main, "Crest fixture A count 1")

        desktop.key("ctrl+t")
        desktop.navigate(origin + "/b")
        desktop.title(main, "Crest fixture B")
        desktop.key("ctrl+Tab")
        desktop.title(main, "Crest fixture A count 1")

        # Menu is the rightmost toolbar control on the fixed Xvfb viewport.
        desktop.click(1086, 69)
        desktop.key("Down", "Return")
        until(
            "vertical layout to persist",
            lambda: profile(directory).get("settings", {}).get("tab_layout") == "vertical",
        )
        desktop.focus(main)
        desktop.click(540, 300)
        desktop.key("j")
        desktop.title(main, "Crest fixture A count 2")

        desktop.key("n")
        desktop.title(main, "Crest fixture popup")
        desktop.key("ctrl+w")
        desktop.title(main, "Crest fixture B")
        desktop.key("ctrl+Tab")
        desktop.title(main, "Crest fixture A count 2")

        desktop.key("p")
        desktop.title(main, "Geolocation error 1")
        until(
            "Chromium child PID namespace and seccomp filter",
            lambda: sandboxed_process(directory / "profile" / "chromium"),
        )
        desktop.key("d")
        dialog = desktop.window("^(All Files|Save File|Save As)$")
        desktop.focus(dialog)
        download = directory / "downloads" / "crest-smoke.txt"
        desktop.key("ctrl+a")
        desktop.xdo("type", str(download))
        desktop.key("Return")
        until("downloaded attachment", lambda: download.exists() and download.read_bytes() == PAYLOAD)

        desktop.focus(main)
        desktop.click(540, 300)
        desktop.key("ctrl+shift+i")
        desktop.window("^DevTools")
        desktop.close(main, process)

    saved = profile(directory)
    assert saved["settings"]["tab_layout"] == "vertical"
    assert [page["url"] for page in saved["tabs"]] == [origin + "/a", origin + "/b"]
    assert [page["url"] for page in saved["bookmarks"]] == [origin + "/a"]
    assert {page["url"] for page in saved["history"]} >= {origin + "/a", origin + "/b"}

    with launch(binary, env, directory) as process:
        main = desktop.window("^Crest Browser")
        desktop.title(main, "Crest fixture A")
        desktop.focus(main)
        desktop.key("ctrl+Tab")
        desktop.title(main, "Crest fixture B")
        subprocess.run(
            [
                str(binary),
                "--profile",
                str(directory / "profile"),
                origin + "/popup",
            ],
            env=env,
            check=True,
            capture_output=True,
            timeout=10,
        )
        desktop.title(main, "Crest fixture popup")
        desktop.close(main, process)


def main():
    for tool in ("Xvfb", "xdotool", "dbus-run-session"):
        if shutil.which(tool) is None:
            raise SystemExit(f"Missing desktop test tool {tool}")
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/crest-browser").resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="crest-smoke-") as temp, http.server.ThreadingHTTPServer(
        ("127.0.0.1", 0), Fixture
    ) as fixture:
        directory = Path(temp)
        for name in ("config", "runtime", "data", "downloads"):
            (directory / name).mkdir(mode=0o700)
        (directory / "config" / "user-dirs.dirs").write_text(
            f'XDG_DOWNLOAD_DIR="{directory}/downloads"\n'
        )
        env = {
            **os.environ,
            "XDG_SESSION_TYPE": "x11",
            "XDG_CURRENT_DESKTOP": "X-Generic",
            "XDG_RUNTIME_DIR": str(directory / "runtime"),
            "XDG_CONFIG_HOME": str(directory / "config"),
            "XDG_DATA_HOME": str(directory / "data"),
            "LIBGL_ALWAYS_SOFTWARE": "1",
        }
        for name in ("WAYLAND_DISPLAY", "XAUTHORITY", "DBUS_SESSION_BUS_ADDRESS"):
            env.pop(name, None)
        thread = threading.Thread(target=fixture.serve_forever, daemon=True)
        thread.start()
        try:
            with private_display(env) as display:
                smoke(binary, directory, display, f"http://127.0.0.1:{fixture.server_port}")
        finally:
            fixture.shutdown()
            thread.join(timeout=5)
    print("Crest Browser desktop smoke test passed")


if __name__ == "__main__":
    main()
