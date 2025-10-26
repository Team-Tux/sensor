import asyncio

import websockets
import json
import matplotlib.pyplot as plt
import matplotlib

matplotlib.use('TkAgg')

WS_SENSORS = "ws://localhost:8080/api/sensors/ws"
WS_TRILATERATIONS = "ws://localhost:8080/api/trilaterations/ws"

sensor_positions = {}
trilateration_positions = {}

plt.ion()

fig, ax = plt.subplots(figsize=(10, 10))

sensor_plot, = ax.plot([], [], 'bo', markersize=12, label='Sensors')
device_plot, = ax.plot([], [], 'rx', markersize=10, mew=2, label='Trilaterations')

async def connect_to_sensors():
    global sensor_positions

    while True:
        async with websockets.connect(WS_SENSORS) as ws:
            print("Connected to sensors websocket")

            async for msg in ws:
                sensor_positions = {i['id']: (i['x'], i['y']) for i in json.loads(msg)}


async def connect_to_trilaterations():
    global trilateration_positions

    while True:
        async with websockets.connect(WS_TRILATERATIONS) as ws:
            print("Connected to trilaterations websocket")

            async for msg in ws:
                trilateration_positions = {i['fingerprint']: (i['x'], i['y']) for i in json.loads(msg)}


def update_plot():
    if sensor_positions:
        s_x = [pos[0] for pos in sensor_positions.values()]
        s_y = [pos[1] for pos in sensor_positions.values()]
        sensor_plot.set_data(s_x, s_y)

    if trilateration_positions:
        t_x = [pos[0] for pos in trilateration_positions.values()]
        t_y = [pos[1] for pos in trilateration_positions.values()]
        device_plot.set_data(t_x, t_y)

    ax.relim()
    ax.autoscale_view()

    fig.canvas.draw()
    fig.canvas.flush_events()


async def main():
    task1 = asyncio.create_task(connect_to_sensors())
    task2 = asyncio.create_task(connect_to_trilaterations())

    ax.set_xlabel("X (in m)")
    ax.set_ylabel("Y (in m)")

    ax.set_title("WiFi Trilateration")
    ax.grid(True)

    plt.tight_layout()

    while True:
        try:
            update_plot()

            await asyncio.sleep(0.05)
        except Exception as e:
            print(e)

            task1.cancel()
            task2.cancel()

            break



if __name__ == "__main__":
    asyncio.run(main())
