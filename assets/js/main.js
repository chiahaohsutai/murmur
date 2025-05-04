const startBtn = document.querySelector("#start");
const stopBtn = document.querySelector("#stop");
const transcription = document.querySelector("#transcription");

let audioCtx, ws, workletNode, ms, sendInterval;
let recording = false;

function float32ToBytes(float32) {
  const buffer = new ArrayBuffer(float32.length * 4);
  const view = new DataView(buffer);
  float32.forEach((val, i) => view.setFloat32(i * 4, val, true));
  return buffer;
}

async function startRecording(interval = 5000) {
  const { protocol: proto, host } = location;
  ws = new WebSocket(`${proto.startsWith("https") ? "wss" : "ws"}://${host}/ws`);
  await new Promise((resolve) => ws.onopen = resolve);

  // Using default sample rate of 48kHz (browser default)
  audioCtx = new AudioContext({ sampleRate: 48000 });
  await audioCtx.audioWorklet.addModule("/assets/js/processor.js");

  ms = await navigator.mediaDevices.getUserMedia({ audio: true });
  const source = audioCtx.createMediaStreamSource(ms);

  let audioBuffer = [];

  workletNode = new AudioWorkletNode(audioCtx, "processor");
  workletNode.port.onmessage = (e) => audioBuffer.push(e.data);

  sendInterval = setInterval(() => {
    if (audioBuffer.length > 0) {
      const len = audioBuffer.reduce((sum, arr) => sum + arr.length, 0);
      const merged = new Float32Array(len);
      let offset = 0;
      for (const chunk of audioBuffer) {
        merged.set(chunk, offset);
        offset += chunk.length;
      }
      ws.send(float32ToBytes(merged));
      audioBuffer = [];
    }
  }, interval)

  source.connect(workletNode).connect(audioCtx.destination);
}

async function stopRecording() {
  if (workletNode) workletNode.port.close();
  if (ms) ms.getTracks().forEach((track) => track.stop());
  if (audioCtx) audioCtx.close();
  if (ws) ws.close();
  if (sendInterval) clearInterval(sendInterval);
}

startBtn.addEventListener("click", async () => {
  if (!recording) {
    recording = true;
    startBtn.innerText = "Listening...";
    startBtn.disabled = true;
    stopBtn.disabled = false;
    await startRecording();
    ws.onmessage = (e) => {
      console.log(JSON.stringify(e, null, 2));
      const contents = transcription.textContent ?? "";
      transcription.textContent = contents.concat(" ").concat(e.data);
    };
  }
});

stopBtn.addEventListener("click", async () => {
  if (recording) {
    recording = false;
    startBtn.disabled = false;
    stopBtn.disabled = true;
    await stopRecording();
  }
});
