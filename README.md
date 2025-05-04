# Murmur

**Murmur** is a speech-to-text web application built with **Actix-Web**, **Askama**, and **Whisper-rs**. It provides web-based audio transcription powered by OpenAI’s Whisper model.

This project is inspired by the ideas in [_Hypermedia Systems_](https://hypermedia.systems/), with a focus on building fully server-driven web applications. I'm particularly interested in **low-JS development patterns** — moving away from single-page applications (SPAs) — and embracing approaches that rely on minimal, lightweight JavaScript.

With Murmur, I aimed to explore how modern web applications can be built efficiently using Rust's performance, safety, and robust web ecosystem, without depending on heavy frontend frameworks.

# Local Installation

You will need to download the whisper model from [Huggingface](https://huggingface.co/ggerganov/whisper.cpp/tree/main) or your preferred platform. During development, I used `ggml-large-v3-turbo`; but your free to chose any of the models. After downloading the model, move the `.bin` file to a directly called `models` under the project root

```bash
mv ggml-large-v3-turbo.bin ../models/ggml-large-v3-turbo.bin
```

## Run the project locally

```bash
# Clone the repository
git clone <repository-url>

# Change directory to the project root
cd murmur

# Install node dependencies
npm install

# Generate tailwind styles (add --watch flag during development)
npx tailwindcss -i ./assets/styles/tailwind.css -o ./assets/styles/output.css

# Start the web server
cargo run
```
