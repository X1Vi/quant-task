import asyncio
import time
import os

SERVER_HOST = os.getenv("SERVER_HOST", "127.0.0.1")
SERVER_PORT = 8080

async def main():
    print(f"Connecting to {SERVER_HOST}:{SERVER_PORT}...")
    while True:
        try:
            reader, writer = await asyncio.open_connection(SERVER_HOST, SERVER_PORT)
            print("Connected to TCP stream")
            count = 0
            start = time.time()
            while True:
                line = await reader.readline()
                if not line:
                    break
                count += 1
                if count % 10000 == 0:
                    elapsed = time.time() - start
                    rate = count / elapsed if elapsed > 0 else 0
                    print(f"Received {count} msgs | {rate:.0f} msg/s")
        except Exception as e:
            print(f"Connection error: {e}, retrying in 2s...")
            await asyncio.sleep(2)

if __name__ == "__main__":
    asyncio.run(main())
