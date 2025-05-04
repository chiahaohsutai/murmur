const startBtn = document.querySelector("#start");
const stopBtn = document.querySelector("#stop");
const transcription = document.querySelector("#transcription");

let audioCtx, ws, workletNode, mms, smm, sendInterval;
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

  mms = await navigator.mediaDevices.getUserMedia({ audio: true });
  const micSource = audioCtx.createMediaStreamSource(mms);

  smm = await navigator.mediaDevices.getDisplayMedia({ video: true, audio: true });
  const sysSource = audioCtx.createMediaStreamSource(smm);

  const merger = audioCtx.createChannelMerger(1);
  micSource.connect(merger, 0, 0);
  sysSource.connect(merger, 0, 0);

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

  merger.connect(workletNode).connect(audioCtx.destination);
}

async function stopRecording() {
  if (workletNode) workletNode.port.close();
  if (mms) mms.getTracks().forEach((track) => track.stop());
  if (audioCtx) audioCtx.close();
  if (ws) ws.close();
  if (sendInterval) clearInterval(sendInterval);
}

startBtn.addEventListener("click", async () => {
  if (!recording) {
    recording = true;
    startBtn.innerText = "Listening...";
    startBtn.classList.add("animate-pulse");
    startBtn.disabled = true;
    stopBtn.disabled = false;
    await startRecording(10000);
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
    startBtn.innerText = "Transcribe";
    startBtn.classList.remove("animate-pulse");
    stopBtn.disabled = true;
    await stopRecording();
  }
});
