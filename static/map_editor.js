const canvas = document.getElementById('editorCanvas');
const ctx = canvas.getContext('2d');

let currentTool = 'WALL'; // 'WALL' หรือ 'POSE'
let isDrawing = false;
let startX = 0, startY = 0;

let virtualWalls = []; // [{ startX, startY, endX, endY }, ...] ในระบบ Pixel
let poses = [];        // [{ x, y, name }, ...] ในระบบ Pixel

// โหลดรูปภาพแผนที่
const mapImage = new Image();
mapImage.src = '/uploads/maps/DA100626_map.png'; // เปลี่ยนตามชื่อภาพแผนที่ของคุณ
mapImage.onload = () => redrawCanvas();

function setTool(tool) {
    currentTool = tool;
    document.getElementById('tool-wall').classList.toggle('active', tool === 'WALL');
    document.getElementById('tool-pose').classList.toggle('active', tool === 'POSE');
}

// 🟢 แปลง Pixel Coordinate บน Canvas เป็น World Coordinate (เมตร)
function pixelToWorld(px, py) {
    const res = parseFloat(document.getElementById('resolution').value);
    const origX = parseFloat(document.getElementById('originX').value);
    const origY = parseFloat(document.getElementById('originY').value);

    const worldX = (px * res) + origX;
    const worldY = ((canvas.height - py) * res) + origY;
    return { x: worldX, y: worldY };
}

// 🟢 แปลง World Coordinate (เมตร) กลับเป็น Pixel Coordinate
function worldToPixel(wx, wy) {
    const res = parseFloat(document.getElementById('resolution').value);
    const origX = parseFloat(document.getElementById('originX').value);
    const origY = parseFloat(document.getElementById('originY').value);

    const px = (wx - origX) / res;
    const py = canvas.height - ((wy - origY) / res);
    return { x: px, y: py };
}

// -------------------------------------------------------------
// CANVAS EVENT LISTENERS (การวาดเส้นและปักจุด)
// -------------------------------------------------------------

canvas.addEventListener('mousedown', (e) => {
    const rect = canvas.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const clickY = e.clientY - rect.top;

    if (currentTool === 'WALL') {
        isDrawing = true;
        startX = clickX;
        startY = clickY;
    } else if (currentTool === 'POSE') {
        const worldPos = pixelToWorld(clickX, clickY);
        const poseName = prompt("ตั้งชื่อ Station/Pose:", `Station_${poses.length + 1}`);
        if (poseName) {
            poses.push({ px: clickX, py: clickY, worldX: worldPos.x, worldY: worldPos.y, name: poseName });
            log(`Added Pose: ${poseName} at (${worldPos.x.toFixed(2)}, ${worldPos.y.toFixed(2)})`);
            redrawCanvas();
        }
    }
});

canvas.addEventListener('mousemove', (e) => {
    if (!isDrawing || currentTool !== 'WALL') return;

    const rect = canvas.getBoundingClientRect();
    const currentX = e.clientX - rect.left;
    const currentY = e.clientY - rect.top;

    redrawCanvas();
    // วาดเส้นตัวอย่างขณะกำลังลาก
    ctx.beginPath();
    ctx.moveTo(startX, startY);
    ctx.lineTo(currentX, currentY);
    ctx.strokeStyle = '#ff3366';
    ctx.lineWidth = 3;
    ctx.stroke();
});

canvas.addEventListener('mouseup', (e) => {
    if (!isDrawing || currentTool !== 'WALL') return;
    isDrawing = false;

    const rect = canvas.getBoundingClientRect();
    const endX = e.clientX - rect.left;
    const endY = e.clientY - rect.top;

    virtualWalls.push({ startX, startY, endX, endY });
    redrawCanvas();
});

function redrawCanvas() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.drawImage(mapImage, 0, 0, canvas.width, canvas.height);

    // วาดเส้น Virtual Walls ทั้งหมด
    virtualWalls.forEach(wall => {
        ctx.beginPath();
        ctx.moveTo(wall.startX, wall.startY);
        ctx.lineTo(wall.endX, wall.endY);
        ctx.strokeStyle = '#dc3545';
        ctx.lineWidth = 3;
        ctx.stroke();
    });

    // วาดหมุด Poses ทั้งหมด
    poses.forEach(p => {
        ctx.beginPath();
        ctx.arc(p.px, p.py, 6, 0, 2 * Math.PI);
        ctx.fillStyle = '#00e676';
        ctx.fill();
        ctx.fillStyle = '#ffffff';
        ctx.font = '12px Arial';
        ctx.fillText(p.name, p.px + 8, p.py + 4);
    });

    document.getElementById('wall-count').innerText = virtualWalls.length;
    document.getElementById('pose-count').innerText = poses.length;
}

function clearCurrentCanvas() {
    virtualWalls = [];
    poses = [];
    redrawCanvas();
    log('Cleared drawn lines and poses.');
}

// -------------------------------------------------------------
// REST API INTEGRATION (การสื่อสารกับ Rust Backend)
// -------------------------------------------------------------

async function saveToDatabase() {
    const mapId = document.getElementById('mapId').value;

    // แปลงข้อมูลเส้น Virtual Walls เป็น World Coordinates ก่อนส่งไป DB
    const formattedAnnotations = virtualWalls.map((wall, index) => {
        const startWorld = pixelToWorld(wall.startX, wall.startY);
        const endWorld = pixelToWorld(wall.endX, wall.endY);
        return {
            map_id: parseInt(mapId),
            name: `Virtual_Wall_${index + 1}`,
            line_type: "VIRTUAL_WALL",
            start_x: startWorld.x,
            start_y: startWorld.y,
            end_x: endWorld.x,
            end_y: endWorld.y
        };
    });

    try {
        // 1. บันทึกเส้น Virtual Walls
        const annResponse = await fetch(`/api/maps/${mapId}/annotations`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(formattedAnnotations)
        });

        if (annResponse.ok) {
            log('✅ Virtual Walls saved to Database successfully!');
        }

        // 2. บันทึก Poses
        for (const pose of poses) {
            await fetch(`/api/maps/${mapId}/poses`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    map_id: parseInt(mapId),
                    name: pose.name,
                    x: pose.worldX,
                    y: pose.worldY,
                    yaw: 0.0
                })
            });
        }
        log('✅ Poses saved to Database successfully!');
    } catch (err) {
        log(`❌ Error saving to DB: ${err.message}`);
    }
}

async function loadMapData() {
    const mapId = document.getElementById('mapId').value;
    try {
        const res = await fetch(`/api/maps/${mapId}/annotations`);
        if (res.ok) {
            const data = await res.json();
            // แปลง World Frame กลับมาเป็น Pixel เพื่อวาดบน Canvas
            virtualWalls = data.map(ann => {
                const startPix = worldToPixel(ann.start_x, ann.start_y);
                const endPix = worldToPixel(ann.end_x, ann.end_y);
                return { startX: startPix.x, startY: startPix.y, endX: endPix.x, endY: endPix.y };
            });
            redrawCanvas();
            log(`Loaded ${data.length} Virtual Walls from Database.`);
        }
    } catch (err) {
        log(`❌ Failed to load annotations: ${err.message}`);
    }
}

function log(msg) {
    const logBox = document.getElementById('log-output');
    logBox.innerHTML = `> ${msg}<br>` + logBox.innerHTML;
}