// (function() {
//     const canvas = document.getElementById('neural-canvas');
//     if (!canvas) return;
//     const ctx = canvas.getContext('2d');

//     let W, H, nodes = [];
//     const COUNT = 60;
//     const MAX_DIST = 160;
//     const COLOR = '0, 217, 255';

//     function resize() {
//         W = canvas.width  = canvas.offsetWidth;
//         H = canvas.height = canvas.offsetHeight;
//     }

//     function initNodes() {
//         nodes = [];
//         for (let i = 0; i < COUNT; i++) {
//             nodes.push({
//                 x: Math.random() * W,
//                 y: Math.random() * H,
//                 vx: (Math.random() - 0.5) * 0.6,
//                 vy: (Math.random() - 0.5) * 0.6,
//                 r: Math.random() * 2 + 1.5
//             });
//         }
//     }

//     function draw() {
//         ctx.clearRect(0, 0, W, H);

//         for (let i = 0; i < nodes.length; i++) {
//             for (let j = i + 1; j < nodes.length; j++) {
//                 const dx = nodes[i].x - nodes[j].x;
//                 const dy = nodes[i].y - nodes[j].y;
//                 const dist = Math.sqrt(dx*dx + dy*dy);
//                 if (dist < MAX_DIST) {
//                     const alpha = (1 - dist / MAX_DIST) * 0.4;
//                     ctx.beginPath();
//                     ctx.strokeStyle = `rgba(${COLOR}, ${alpha})`;
//                     ctx.lineWidth = 0.8;
//                     ctx.moveTo(nodes[i].x, nodes[i].y);
//                     ctx.lineTo(nodes[j].x, nodes[j].y);
//                     ctx.stroke();
//                 }
//             }
//         }

//         for (const n of nodes) {
//             ctx.beginPath();
//             ctx.arc(n.x, n.y, n.r, 0, Math.PI * 2);
//             ctx.fillStyle = `rgba(${COLOR}, 0.7)`;
//             ctx.fill();
//         }
//     }

//     function update() {
//         for (const n of nodes) {
//             n.x += n.vx;
//             n.y += n.vy;
//             if (n.x < 0 || n.x > W) n.vx *= -1;
//             if (n.y < 0 || n.y > H) n.vy *= -1;
//         }
//     }

//     function loop() {
//         update();
//         draw();
//         requestAnimationFrame(loop);
//     }

//     window.addEventListener('resize', () => { resize(); initNodes(); });
//     resize();
//     initNodes();
//     loop();
// })();

// ВАРИАНТ 2 НАчало

// (function () {
//   const canvas = document.getElementById("neural-canvas");
//   if (!canvas) return;
//   const ctx = canvas.getContext("2d");

//   let W,
//     H,
//     nodes = [];
//   const COUNT = Math.floor(Math.random() * 60) + 60; // 60-120 точек случайно
//   const MAX_DIST = Math.floor(Math.random() * 80) + 140; // 140-220 дистанция случайно

//   let hue = 0;

//   function resize() {
//     W = canvas.width = canvas.offsetWidth;
//     H = canvas.height = canvas.offsetHeight;
//   }

//   function initNodes() {
//     nodes = [];
//     for (let i = 0; i < COUNT; i++) {
//       nodes.push({
//         x: Math.random() * W,
//         y: Math.random() * H,
//         vx: (Math.random() - 0.5) * 0.6,
//         vy: (Math.random() - 0.5) * 0.6,
//         r: Math.random() * 2 + 1.5,
//         hueOffset: Math.random() * 360,
//       });
//     }
//   }

//   function draw() {
//     ctx.clearRect(0, 0, W, H);
//     hue = (hue + 0.5) % 360;

//     for (let i = 0; i < nodes.length; i++) {
//       for (let j = i + 1; j < nodes.length; j++) {
//         const dx = nodes[i].x - nodes[j].x;
//         const dy = nodes[i].y - nodes[j].y;
//         const dist = Math.sqrt(dx * dx + dy * dy);
//         if (dist < MAX_DIST) {
//           const alpha = (1 - dist / MAX_DIST) * 0.5;
//           const lineHue =
//             (hue + (nodes[i].hueOffset + nodes[j].hueOffset) / 2) % 360;

//           const grad = ctx.createLinearGradient(
//             nodes[i].x,
//             nodes[i].y,
//             nodes[j].x,
//             nodes[j].y,
//           );
//           grad.addColorStop(0, `hsla(${lineHue}, 100%, 65%, ${alpha})`);
//           grad.addColorStop(
//             0.5,
//             `hsla(${(lineHue + 60) % 360}, 100%, 65%, ${alpha})`,
//           );
//           grad.addColorStop(
//             1,
//             `hsla(${(lineHue + 120) % 360}, 100%, 65%, ${alpha})`,
//           );

//           const cpx1 =
//             (nodes[i].x + nodes[j].x) / 2 + (nodes[j].y - nodes[i].y) * 0.3;
//           const cpy1 =
//             (nodes[i].y + nodes[j].y) / 2 - (nodes[j].x - nodes[i].x) * 0.3;
//           const cpx2 =
//             (nodes[i].x + nodes[j].x) / 2 - (nodes[j].y - nodes[i].y) * 0.3;
//           const cpy2 =
//             (nodes[i].y + nodes[j].y) / 2 + (nodes[j].x - nodes[i].x) * 0.3;

//           ctx.beginPath();
//           ctx.strokeStyle = grad;
//           ctx.lineWidth = 0.8;
//           ctx.moveTo(nodes[i].x, nodes[i].y);
//           ctx.bezierCurveTo(cpx1, cpy1, cpx2, cpy2, nodes[j].x, nodes[j].y);
//           ctx.stroke();
//         }
//       }
//     }

//     for (const n of nodes) {
//       const nodeHue = (hue + n.hueOffset) % 360;
//       ctx.beginPath();
//       ctx.arc(n.x, n.y, n.r, 0, Math.PI * 2);
//       ctx.fillStyle = `hsla(${nodeHue}, 100%, 70%, 0.85)`;
//       ctx.fill();
//     }
//   }

//   function update() {
//     for (const n of nodes) {
//       n.x += n.vx;
//       n.y += n.vy;
//       if (n.x < 0 || n.x > W) n.vx *= -1;
//       if (n.y < 0 || n.y > H) n.vy *= -1;
//     }
//   }

//   function loop() {
//     update();
//     draw();
//     requestAnimationFrame(loop);
//   }

//   window.addEventListener("resize", () => {
//     resize();
//     initNodes();
//   });
//   resize();
//   initNodes();
//   loop();
// })();
// Вариант 2 конец

// Вариант3 Начало

(function() {
    const canvas = document.getElementById('neural-canvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');

    let W, H, nodes = [];
    let mouse = { x: -9999, y: -9999 };
    const COUNT = Math.floor(Math.random() * 60) + 60;
    const MAX_DIST = Math.floor(Math.random() * 80) + 140;
    const MOUSE_RADIUS = 180;
    const MOUSE_FORCE = 0.06;

    let hue = 0;
    let time = 0;

    canvas.addEventListener('mousemove', e => {
        const rect = canvas.getBoundingClientRect();
        mouse.x = e.clientX - rect.left;
        mouse.y = e.clientY - rect.top;
    });

    canvas.addEventListener('mouseleave', () => {
        mouse.x = -9999;
        mouse.y = -9999;
    });

    function resize() {
        W = canvas.width  = canvas.offsetWidth;
        H = canvas.height = canvas.offsetHeight;
    }

    function initNodes() {
        nodes = [];
        for (let i = 0; i < COUNT; i++) {
            nodes.push({
                x: Math.random() * W,
                y: Math.random() * H,
                vx: (Math.random() - 0.5) * 0.6,
                vy: (Math.random() - 0.5) * 0.6,
                r: Math.random() * 2 + 1.5,
                baseR: 0,
                pulseOffset: Math.random() * Math.PI * 2,
                hueOffset: Math.random() * 360
            });
        }
        nodes.forEach(n => n.baseR = n.r);
    }

    function draw() {
        ctx.clearRect(0, 0, W, H);
        hue = (hue + 0.5) % 360;
        time += 0.03;

        for (let i = 0; i < nodes.length; i++) {
            for (let j = i + 1; j < nodes.length; j++) {
                const dx = nodes[i].x - nodes[j].x;
                const dy = nodes[i].y - nodes[j].y;
                const dist = Math.sqrt(dx*dx + dy*dy);
                if (dist < MAX_DIST) {
                    const alpha = (1 - dist / MAX_DIST) * 0.5;
                    const lineHue = (hue + (nodes[i].hueOffset + nodes[j].hueOffset) / 2) % 360;

                    const grad = ctx.createLinearGradient(nodes[i].x, nodes[i].y, nodes[j].x, nodes[j].y);
                    grad.addColorStop(0, `hsla(${lineHue}, 100%, 65%, ${alpha})`);
                    grad.addColorStop(0.5, `hsla(${(lineHue + 60) % 360}, 100%, 65%, ${alpha})`);
                    grad.addColorStop(1, `hsla(${(lineHue + 120) % 360}, 100%, 65%, ${alpha})`);

                    const cpx1 = (nodes[i].x + nodes[j].x) / 2 + (nodes[j].y - nodes[i].y) * 0.3;
                    const cpy1 = (nodes[i].y + nodes[j].y) / 2 - (nodes[j].x - nodes[i].x) * 0.3;
                    const cpx2 = (nodes[i].x + nodes[j].x) / 2 - (nodes[j].y - nodes[i].y) * 0.3;
                    const cpy2 = (nodes[i].y + nodes[j].y) / 2 + (nodes[j].x - nodes[i].x) * 0.3;

                    ctx.beginPath();
                    ctx.strokeStyle = grad;
                    ctx.lineWidth = 0.8;
                    ctx.moveTo(nodes[i].x, nodes[i].y);
                    ctx.bezierCurveTo(cpx1, cpy1, cpx2, cpy2, nodes[j].x, nodes[j].y);
                    ctx.stroke();
                }
            }
        }

        for (const n of nodes) {
            const pulse = Math.sin(time + n.pulseOffset) * 0.8;
            const nodeHue = (hue + n.hueOffset) % 360;
            ctx.beginPath();
            ctx.arc(n.x, n.y, n.baseR + pulse, 0, Math.PI * 2);
            ctx.fillStyle = `hsla(${nodeHue}, 100%, 70%, 0.85)`;
            ctx.fill();
        }
    }

    function update() {
        for (const n of nodes) {
            const dx = mouse.x - n.x;
            const dy = mouse.y - n.y;
            const dist = Math.sqrt(dx*dx + dy*dy);

            if (dist < MOUSE_RADIUS && dist > 0) {
                const force = (1 - dist / MOUSE_RADIUS) * MOUSE_FORCE;
                n.vx += dx / dist * force;
                n.vy += dy / dist * force;
            }

            const speed = Math.sqrt(n.vx*n.vx + n.vy*n.vy);
            if (speed > 2) {
                n.vx = (n.vx / speed) * 2;
                n.vy = (n.vy / speed) * 2;
            }

            n.x += n.vx;
            n.y += n.vy;

            if (n.x < 0 || n.x > W) n.vx *= -1;
            if (n.y < 0 || n.y > H) n.vy *= -1;
        }
    }

    function loop() {
        update();
        draw();
        requestAnimationFrame(loop);
    }

    window.addEventListener('resize', () => { resize(); initNodes(); });
    resize();
    initNodes();
    loop();
})();

