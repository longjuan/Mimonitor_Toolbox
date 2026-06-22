import json
import os
import tempfile
import threading
import time
import unittest
from unittest import mock

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import monitor_controller as app


class FakePopen:
    active = 0
    max_active = 0
    created = 0
    state_lock = threading.Lock()
    return_code = 0
    stderr = ""

    @classmethod
    def reset(cls):
        with cls.state_lock:
            cls.active = 0
            cls.max_active = 0
            cls.created = 0
            cls.return_code = 0
            cls.stderr = ""

    def __init__(self, *args, **kwargs):
        self.returncode = None
        with self.state_lock:
            type(self).created += 1
            type(self).active += 1
            type(self).max_active = max(type(self).max_active, type(self).active)

    def communicate(self, timeout=None):
        time.sleep(0.01)
        with self.state_lock:
            type(self).active -= 1
        self.returncode = type(self).return_code
        return ("ok" if self.returncode == 0 else "", type(self).stderr)

    def kill(self):
        self.returncode = -9

    def poll(self):
        return self.returncode


class AdbRuntimeTests(unittest.TestCase):
    def setUp(self):
        FakePopen.reset()
        app.unblock_adb_spawns()
        with app._adb_process_lock:
            app._adb_processes.clear()

    def tearDown(self):
        app.unblock_adb_spawns()

    def test_adb_processes_are_serialized(self):
        with mock.patch.object(app.subprocess, "Popen", FakePopen):
            workers = [threading.Thread(target=app.adb_run, args=(["version"],)) for _ in range(12)]
            for worker in workers:
                worker.start()
            for worker in workers:
                worker.join(timeout=3)
                self.assertFalse(worker.is_alive())

        self.assertEqual(FakePopen.created, 12)
        self.assertEqual(FakePopen.max_active, 1)

    def test_nested_adb_transaction_is_reentrant(self):
        with mock.patch.object(app.subprocess, "Popen", FakePopen):
            with app.Adb("127.0.0.1").transaction():
                self.assertEqual(app.adb_run(["version"], check=True), "ok")

    def test_shutdown_block_prevents_new_process(self):
        with mock.patch.object(app.subprocess, "Popen", FakePopen):
            app.block_adb_spawns()
            with self.assertRaisesRegex(RuntimeError, "正在退出"):
                app.adb_run(["version"], check=True)
        self.assertEqual(FakePopen.created, 0)

    def test_strict_adb_failure_raises(self):
        FakePopen.return_code = 7
        FakePopen.stderr = "device offline"
        with mock.patch.object(app.subprocess, "Popen", FakePopen):
            with self.assertRaisesRegex(RuntimeError, "device offline"):
                app.adb_run(["version"], check=True)

    def test_adb_server_probe_does_not_start_adb(self):
        fake_socket = mock.Mock()
        with mock.patch.object(app.socket, "create_connection", return_value=fake_socket) as connect:
            self.assertTrue(app.is_adb_server_alive())
        connect.assert_called_once_with(("127.0.0.1", int(app.ADB_SERVER_PORT)), timeout=0.2)
        fake_socket.close.assert_called_once_with()

        with mock.patch.object(app.socket, "create_connection", side_effect=ConnectionRefusedError):
            self.assertFalse(app.is_adb_server_alive())

    def test_dead_adb_server_is_restarted_and_device_reconnected(self):
        events = []
        commands = []

        class FakeSignal:
            def emit(self, *args):
                events.append(args)

        class FakeApp:
            _cleanup_done = False
            _windows_session_ending = False
            adb_connected = True
            adb = type("FakeAdb", (), {"ip": "192.168.5.205"})()
            _adb_server_monitor_checking = False
            _adb_server_retry_after = 0.0
            adb_server_event = FakeSignal()

        def fake_adb_run(args, timeout=10, check=False):
            commands.append((args, timeout, check))
            return "connected to 192.168.5.205:5555"

        fake = FakeApp()
        with mock.patch.object(app, "is_adb_server_alive", side_effect=[False, True]), \
                mock.patch.object(app, "adb_run", side_effect=fake_adb_run), \
                mock.patch.object(app, "async_run", side_effect=lambda fn: fn()):
            app.App._monitor_adb_server(fake)

        self.assertTrue(fake._adb_server_monitor_checking)
        self.assertEqual(events, [
            ("restarting", ""),
            ("recovered", "connected to 192.168.5.205:5555"),
        ])
        self.assertEqual(commands[0], (["start-server"], 5, True))
        self.assertEqual(commands[1], (["connect", "192.168.5.205:5555"], 5, False))

    def test_adb_restart_notification_is_not_repeated(self):
        messages = []
        hud = []
        logs = []

        class FakeTray:
            def showMessage(self, *args):
                messages.append(args)

        class FakeOsd:
            def show_hud(self, *args):
                hud.append(args)

        class FakeApp:
            _cleanup_done = False
            _adb_server_monitor_checking = True
            _adb_server_down_notified = False
            _adb_server_failure_notified = False
            _adb_server_retry_after = 0.0
            tray_icon = FakeTray()
            osd = FakeOsd()

            def log(self, message):
                logs.append(message)

        fake = FakeApp()
        app.App._on_adb_server_event(fake, "restarting", "")
        app.App._on_adb_server_event(fake, "restarting", "")

        self.assertEqual(len(messages), 1)
        self.assertEqual(hud, [("ADB 进程", "正在重启")])
        self.assertEqual(logs, ["检测到 ADB 进程被杀死，正在重启"])


class SettingsTests(unittest.TestCase):
    def test_concurrent_updates_are_atomic(self):
        with tempfile.TemporaryDirectory() as folder:
            config_path = os.path.join(folder, "config.json")
            with mock.patch.object(app, "get_settings_path", return_value=config_path):
                results = []

                def update(index):
                    results.append(app.update_settings({f"key_{index}": index}))

                workers = [threading.Thread(target=update, args=(index,)) for index in range(40)]
                for worker in workers:
                    worker.start()
                for worker in workers:
                    worker.join(timeout=3)
                    self.assertFalse(worker.is_alive())

                self.assertTrue(all(results))
                settings = app.load_settings()
                for index in range(40):
                    self.assertEqual(settings[f"key_{index}"], index)

                with open(config_path, "r", encoding="utf-8") as stream:
                    json.load(stream)
                self.assertFalse([name for name in os.listdir(folder) if name.endswith(".tmp")])


class StateMachineTests(unittest.TestCase):
    def test_polled_local_dimming_does_not_update_memory(self):
        class FakeApp:
            def _hdr_memory_enabled(self):
                return True

            def _save_local_dimming_memory(self, memory):
                raise AssertionError("automatic refresh must not save memory")

        app.App._remember_local_dimming_value(FakeApp(), 3, log_change=False)

    def test_hdr_transition_applies_memory_before_refresh(self):
        calls = []

        class FakeApp:
            _hdr_windows_state = True
            _hdr_last_state = False

            def _update_hdr_memory_status_label(self, source=None):
                calls.append(("status", source))

            def _hdr_memory_enabled(self):
                return True

            def _schedule_hdr_memory_apply(self, delay_ms=250):
                calls.append(("apply", delay_ms))

            def _schedule_picture_refresh_after_hdr_change(self, state, initial_delay_ms=0):
                calls.append(("refresh", state, initial_delay_ms))

        fake = FakeApp()
        app.App._reconcile_hdr_memory_state(fake, "Windows HDR")
        self.assertEqual(fake._hdr_last_state, True)
        self.assertEqual(calls[1:], [("apply", 120), ("refresh", True, 300)])

    def test_page_refresh_cleanup_updates_state_in_slot(self):
        hidden = []

        class FakeApp:
            _page_loaded = set()
            _page_loading = {"picturePage"}

            def _hide_loading_overlay(self, page_name):
                hidden.append(page_name)

        fake = FakeApp()
        app.App._finish_page_refresh(fake, "picturePage", True)
        self.assertIn("picturePage", fake._page_loaded)
        self.assertNotIn("picturePage", fake._page_loading)
        self.assertEqual(hidden, ["picturePage"])

    def test_page_refresh_reads_one_adb_transaction(self):
        emitted = []

        class FakeSignal:
            def __init__(self, name):
                self.name = name

            def emit(self, *args):
                emitted.append((self.name, args))

        class FakeTransaction:
            def __init__(self, adb):
                self.adb = adb

            def __enter__(self):
                self.adb.depth += 1

            def __exit__(self, exc_type, exc_value, traceback):
                self.adb.depth -= 1

        class FakeAdb:
            depth = 0

            def transaction(self):
                return FakeTransaction(self)

            def shell(self, cmd):
                if self.depth != 1:
                    raise AssertionError("settings read escaped the page transaction")
                return "picture_mode=2"

            def jni_get(self, key):
                if self.depth != 1:
                    raise AssertionError("JNI read escaped the page transaction")
                return "40"

        class FakeApp:
            adb_connected = True
            _picture_mode_switch_seq = 0
            _page_loading = set()
            _page_data_keys = {
                "picturePage": {
                    "settings": ["picture_mode"],
                    "jni": ["g_disp__disp_back_light"],
                }
            }
            adb = FakeAdb()
            values_signal = FakeSignal("values")
            jni_values_signal = FakeSignal("jni")
            page_refresh_finished = FakeSignal("finished")

            def _mark_adb_busy(self, duration):
                pass

            def _show_loading_overlay(self, page_name):
                pass

            def log(self, message):
                pass

        with mock.patch.object(app, "async_run", side_effect=lambda fn: fn()):
            app.App._refresh_page_data(FakeApp(), "picturePage")

        self.assertEqual(FakeApp.adb.depth, 0)
        self.assertIn(("values", ({"picture_mode": 2},)), emitted)
        self.assertIn(("jni", ({"g_disp__disp_back_light": 40},)), emitted)
        self.assertIn(("finished", ("picturePage", True)), emitted)


if __name__ == "__main__":
    unittest.main()
