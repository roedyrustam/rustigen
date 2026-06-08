# Rust Agentic Chatbot 🤖✨

Aplikasi Chatbot Agen Otonom (Agentic AI) dengan visual antarmuka premium, dibangun menggunakan backend Rust berkinerja tinggi (Axum + Tokio) serta Gemini API sebagai otak penalaran LLM.

## Fitur Utama

1. **Reasoning Loop (Proses Berpikir)**: Agen tidak langsung menjawab, melainkan merumuskan rencana aksi, memanggil alat (tools), menganalisis hasil, dan menyajikan solusi final (mirip mekanisme Deep Thinking).
2. **Akses Alat Otonom (Tools)**:
   - 🧮 **Kalkulator**: Parser matematika presisi untuk mengevaluasi perhitungan.
   - 🖥️ **Informasi Sistem**: Mengakses platform OS, arsitektur CPU, dan uptime server.
   - 📂 **Akses Direktori**: Menelusuri file proyek langsung dari backend secara aman.
   - 🕒 **Waktu & Tanggal**: Mendapatkan waktu sistem secara langsung.
3. **Visual UI Premium & Responsif**:
   - Desain gelap (*dark theme*) modern dengan efek *glassmorphism* dan borders neon yang menakjubkan.
   - Panel samping (*sidebar*) untuk manajemen sesi percakapan.
   - Dropdown untuk menampilkan/menyembunyikan detail langkah berpikir (*Thinking Process*).
   - Pengaturan interaktif untuk mengganti API Key, model Gemini, dan temperatur.
4. **Dukungan Mode Demo**:
   - Jika belum memiliki Gemini API Key, agen tetap dapat dijalankan dengan simulasi otonom realistis untuk kueri matematika, sistem, dan direktori file.

---

## Struktur Proyek

```text
my-ai-agent/
├── Cargo.toml            # Konfigurasi dependensi Rust & profil rilis
├── src/
│   ├── main.rs           # Setup server Axum & router static/API
│   └── agent.rs          # Loop penalaran agen otonom & parser alat
└── static/
    ├── index.html        # Kerangka antarmuka pengguna
    ├── style.css         # Desain premium, transisi, & animasi
    └── app.js            # Logika interaksi frontend & komunikasi API
```

---

## Cara Menjalankan Aplikasi

### Prasyarat
- [Rust](https://www.rust-lang.org/tools/install) (versi 1.80 ke atas direkomendasikan) sudah terpasang di sistem Anda.

### Langkah-Langkah

1. **Buka Terminal** dan arahkan ke folder proyek:
   ```bash
   cd C:\Users\roedy\my-ai-agent
   ```

2. **Jalankan Server**:
   Anda dapat langsung menjalankan server dengan perintah Cargo:
   ```bash
   cargo run
   ```
   *Tip: Jika Anda ingin memuat API Key secara otomatis saat server dinyalakan tanpa memasukkannya di UI, Anda dapat mengekspornya di terminal terlebih dahulu:*
   ```powershell
   $env:GEMINI_API_KEY="AIzaSyYourGeminiApiKeyHere"
   cargo run
   ```

3. **Buka Browser**:
   Setelah server aktif, buka peramban Anda dan kunjungi alamat berikut:
   ```text
   http://localhost:3000
   ```

---

## Menggunakan Aplikasi

- **Mode Demo**: Cobalah mengirim pesan seperti `"Tampilkan info sistem"`, `"Hitung 15 * (14 - 4)"`, atau `"Buka file direktori"` untuk melihat agen mensimulasikan pemikiran otonom dan memanggil perkakas backend.
- **Mode Gemini Riil**:
  1. Klik ikon **Pengaturan (Gir)** ⚙️ di pojok kiri bawah.
  2. Masukkan **Gemini API Key** Anda (didapatkan gratis dari Google AI Studio).
  3. Pilih model yang diinginkan (misalnya `gemini-2.0-flash` atau `gemini-1.5-pro`).
  4. Klik **Simpan Perubahan**. Status badge di atas layar akan berubah menjadi **Terkoneksi**.
  5. Kirim pertanyaan kompleks dan perhatikan agen berpikir secara otonom!

---
*Dibuat dengan ❤️ menggunakan Rust & Gemini API.*
