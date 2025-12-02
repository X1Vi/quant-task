import asyncio
import json
import time


async def main():
    # Retry connection until server is ready
    max_retries = 10
    retry_delay = 1
    
    for attempt in range(max_retries):
        try:
            print(f"🔌 Attempting to connect (attempt {attempt + 1}/{max_retries})...")
            reader, writer = await asyncio.open_connection('127.0.0.1', 8080)
            print("✅ Connected to server")
            break
        except Exception as e:
            if attempt < max_retries - 1:
                print(f"⏳ Server not ready, retrying in {retry_delay}s...")
                await asyncio.sleep(retry_delay)
            else:
                print(f"❌ Connection failed after {max_retries} attempts: {e}")
                return
    
    count = 0
    total = 0
    last_time = time.time()
    
    while True:
        line = await reader.readline()
        if not line:
            print("❌ Connection closed by server")
            break
        
        count += 1
        total += 1
        
        # Log every 100 messages
        if total % 100 == 0:
            print(f"📦 Received {total} total messages")
        
        # Log rate every second
        now = time.time()
        if now - last_time >= 1.0:
            print(f"📊 Rate: {count} msg/s")
            count = 0
            last_time = now


if __name__ == "__main__":
    asyncio.run(main())
