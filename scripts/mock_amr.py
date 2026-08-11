import time
import math
import requests

# ตั้งค่า URL ของ Rust Backend
SERVER_URL = "http://localhost:3000"
AMR_NAME = "AMR-01"

# คุณลักษณะการเคลื่อนที่ของหุ่นยนต์จำลอง
SPEED = 0.1         # ความเร็วเคลื่อนที่ (เมตร/วินาที)
TICK_RATE = 0.1     # อัปเดตพิกัดทุกๆ 0.1 วินาที (10 Hz)
STEP_DIST = SPEED * TICK_RATE

def get_current_pose():
    try:
        res = requests.get(f"{SERVER_URL}/api/amr/{AMR_NAME}/pose")
        if res.status_code == 200:
            return res.json()["pose"]
    except Exception as e:
        print(f"❌ Error fetching pose: {e}")
    return {"x": 0.0, "y": 0.0, "theta": 0.0}

def update_pose(x, y, theta):
    payload = {"x": round(x, 3), "y": round(y, 3), "theta": round(theta, 3)}
    try:
        requests.post(f"{SERVER_URL}/api/amr/{AMR_NAME}/pose", json=payload)
    except Exception as e:
        print(f"❌ Error updating pose: {e}")

def run_simulation():
    print(f"🤖 Mock AMR ({AMR_NAME}) Started!")
    print(f"📡 Connecting to {SERVER_URL} ...\n")

    # กำหนดพิกัดเริ่มต้น และพิกัดเป้าหมายทดสอบ
    curr_pose = get_current_pose()
    curr_x, curr_y = curr_pose["x"], curr_pose["y"]
    
    # กำหนด Waypoints ปลอมให้รถลองวิ่งวน
    waypoints = [
        (0.0, 8.0),
    ]
    
    wp_index = 0

    while True:
        target_x, target_y = waypoints[wp_index]

        dx = target_x - curr_x
        dy = target_y - curr_y
        distance = math.hypot(dx, dy)

        # หากเข้าใกล้เป้าหมายในระยะ 0.05 เมตร ให้เปลี่ยนไป Waypoint ถัดไป
        if distance < 0.05:
            print(f"🎯 Reached Waypoint {wp_index + 1}: ({target_x}, {target_y})")
            wp_index = (wp_index + 1) % len(waypoints)
            time.sleep(1.0) # จอดรอ 1 วินาที
            continue

        # คำนวณมุมมุ่งหน้า (Theta)
        target_theta = math.atan2(dy, dx)

        # ขยับตำแหน่ง X, Y เข้าหาเป้าหมายตามความเร็ว
        curr_x += STEP_DIST * math.cos(target_theta)
        curr_y += STEP_DIST * math.sin(target_theta)

        # ส่งพิกัดใหม่กลับไปยัง Backend
        update_pose(curr_x, curr_y, target_theta)

        print(f"📍 Pos: X={curr_x:.2f}, Y={curr_y:.2f}, Theta={target_theta:.2f} rad", end="\r")
        time.sleep(TICK_RATE)

if __name__ == "__main__":
    run_simulation()