"""Minimal QEMU GDB-RSP controller: enable register logging at W03 entry.

No target memory/register is modified. A debug breakpoint starts logging at
one known ELF symbol. The existing capture owner launches/stops the emulator.
"""
import socket
import time


def enable_at_entry(path, entry, result):
    try:
        with socket.socket(socket.AF_UNIX) as connection:
            connection.settimeout(10)
            deadline = time.monotonic() + 10
            while True:
                try:
                    connection.connect(str(path))
                    break
                except (FileNotFoundError, ConnectionRefusedError):
                    if time.monotonic() >= deadline:
                        raise TimeoutError("QEMU GDB socket unavailable")
                    time.sleep(0.01)

            def send(payload):
                data = payload.encode("ascii")
                connection.sendall(b"$" + data + b"#" + f"{sum(data) % 256:02x}".encode())

            def receive():
                while True:
                    lead = connection.recv(1)
                    if not lead:
                        raise EOFError("QEMU debugger disconnected")
                    if lead != b"$":
                        continue
                    data = bytearray()
                    while True:
                        byte = connection.recv(1)
                        if not byte:
                            raise EOFError("incomplete debugger packet")
                        if byte == b"#":
                            break
                        data.extend(byte)
                    checksum = b""
                    while len(checksum) < 2:
                        part = connection.recv(2 - len(checksum))
                        if not part:
                            raise EOFError("incomplete checksum")
                        checksum += part
                    if int(checksum, 16) != sum(data) % 256:
                        raise ValueError("debugger checksum mismatch")
                    connection.sendall(b"+")
                    answer = data.decode("ascii")
                    if answer.startswith("O") and answer != "OK":
                        continue  # Optional monitor console-output packet.
                    return answer

            def request(payload):
                send(payload)
                return receive()

            if request(f"Z1,{entry:x},4") != "OK":
                raise ValueError("entry breakpoint rejected")
            stopped = request("c")
            if not stopped.startswith(("T05", "S05")):
                raise ValueError("entry breakpoint was not reached: " + stopped)
            registers = bytes.fromhex(request("g"))
            pc = int.from_bytes(registers[32 * 8:33 * 8], "little")
            sp = int.from_bytes(registers[31 * 8:32 * 8], "little")
            if pc != entry:
                raise ValueError(f"unexpected breakpoint PC {pc:x}")
            if request("qRcmd," + b"log cpu,nochain".hex()) != "OK":
                raise ValueError("monitor logging rejected")
            if request(f"z1,{entry:x},4") != "OK":
                raise ValueError("entry breakpoint removal rejected")
            result.update(entry_pc=pc, entry_sp=sp, logging_enabled=True)
            send("c")
    except Exception as error:
        result["error"] = str(error)
