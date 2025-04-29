const { protocol: proto, host } = location;
const wsUri = `${proto.startsWith("https") ? "wss" : "ws"}://${host}/ws`;

const transcription = document.querySelector("#transcription");
const socket = new WebSocket(wsUri);

socket.onmessage = (e) => {
  contents = transcription.textContent ?? "";
  transcription.textContent = contents.concat(" ").concat(e.data);
};

class Transcriber {
  constructor() {
    this.active = false;
    this.micStream = null;
    this.sysStream = null;
    this.recorder = null;
    this.ctx = null;
  }

  async start() {
    this.micStream = await navigator.mediaDevices.getUserMedia({ audio: true });
    this.sysStream = await navigator.mediaDevices.getDisplayMedia({
      audio: true,
    });
    this.ctx = new window.AudioContext({ sampleRate: 16000 });

    const micSource = this.ctx.createMediaStreamSource(this.micStream);
    const sysSource = this.ctx.createMediaStreamSource(this.sysStream);
    const dest = this.ctx.createMediaStreamDestination();

    if (micSource) micSource.connect(dest);
    if (sysSource) sysSource.connect(dest);

    await this.ctx.resume()

    const opts = { mimeType: "audio/webm;codecs=pcm" };
    this.recorder = new MediaRecorder(dest.stream, opts);

    this.recorder.ondataavailable = (e) => {
        if (e.data.size > 0) socket.send(e.data);
    };
    this.recorder.start(5000);
    this.active = true;
  }

  async stop() {
    if (this.recorder && this.recorder.state !== "inactive") {
      this.recorder.stop();
    }
    if (this.micStream) {
      this.micStream.getTracks().forEach((track) => track.stop());
      this.micStream = null;
    }
    if (this.sysStream) {
      this.sysStream.getTracks().forEach((track) => track.stop());
      this.sysStream = null;
    }
    if (this.ctx) {
      await this.ctx.close();
      this.ctx = null;
    }
    this.active = false;
  }
}

const startBtn = document.querySelector("#start");
const stopBtn = document.querySelector("#stop");

const transcriber = new Transcriber();

if (startBtn) {
  startBtn.addEventListener("click", async () => {
    await transcriber.start();
    startBtn.textContent = "Listening...";
  });
}

if (stopBtn) {
  stopBtn.addEventListener("click", () => {
    transcriber.stop();
  });
}
