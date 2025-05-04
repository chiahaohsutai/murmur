class ResampleProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.buffer = []
    this.sampleRate = 48000;
  }

  process(inputs, _outputs, _parameters) {
    const input = inputs[0];
    if (input.length > 0) {
      const left = input[0];
      const right = input[1] ?? input[0];
      const mono = left.map((s, i) => 0.5 * (s + right[i]));
      this.buffer.push(...this.applyLowPass(mono));

      const targetSampleRate = 16000;
      const ratio = this.sampleRate / targetSampleRate;

      const neededSamples = Math.floor(this.buffer.length / ratio);
      if (neededSamples > 0) {
        const resampledBuffer = new Float32Array(neededSamples);
        for (let i = 0; i < neededSamples; i++) {
          resampledBuffer[i] = this.buffer[Math.floor(i * ratio)];
        }
        this.buffer.splice(0, Math.floor(neededSamples * ratio));
        this.port.postMessage(resampledBuffer);
      }
    };
    return true;
  }

  applyLowPass(input, windowSize = 5) {
    const output = new Float32Array(input.length);
    for (let i = 0; i < input.length; i++) {
      let sum = 0;
      for (let j = 0; j < windowSize; j++) {
        if (i - j >= 0) sum += input[i - j];
      }
      output[i] = sum / windowSize;
    }
    return output;
  }
}

registerProcessor("processor", ResampleProcessor);
