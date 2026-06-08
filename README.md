# Rust Agentic Chatbot 🤖✨

Aplikasi Chatbot Agen Otonom (Agentic AI) dengan visual antarmuka premium, dibangun menggunakan backend Rust berkinerja tinggi (Axum + Tokio) serta Gemini API sebagai otak penalaran LLM.

## Fitur Utama

1. **Reasoning Loop (Proses Berpikir)**: Agen tidak langsung menjawab, melainkan merumuskan rencana aksi, memanggil alat (tools), menganalisis hasil, dan menyajikan solusi final (mekanisme Deep Thinking dengan visual proses berpikir yang collapsible).
2. **Akses Alat Otonom Lengkap (Tools)**:
   - 📄 **Manipulasi File (`read_file` & `write_file`)**: Membaca isi file proyek serta membuat/mengubah berkas secara aman langsung di dalam workspace.
   - 🌐 **Pencarian Web & Scrape (`search_web` & `fetch_url`)**: Melakukan pencarian di internet via DuckDuckGo serta mengekstrak teks polos halaman web secara real-time.
   - 🚀 **Auto Post Threads (`post_to_threads`)**: Membuat draf postingan teroptimasi algoritma timeline Threads (hook menarik, spasi rapi, emoji, dan CTA interaktif) dan secara otomatis membukanya di tab peramban baru via Threads Web Intent (tidak membutuhkan setup token API yang rumit).
   - 🖥️ **Informasi Sistem**: Mengakses platform OS, arsitektur CPU, dan uptime server.
   - 🧮 **Kalkulator**: Parser matematika presisi untuk mengevaluasi perhitungan.
   - 🕒 **Waktu & Tanggal**: Mendapatkan waktu sistem secara langsung.
3. **Visual UI Premium & Responsif**:
   - Desain gelap (*dark theme*) modern dengan efek *glassmorphism* dan borders neon yang menakjubkan.
   - Panel samping (*sidebar*) untuk manajemen sesi percakapan.
   - Dropdown untuk menampilkan/menyembunyikan detail langkah berpikir (*Thinking Process*).
   - Tombol **"Salin" (Copy Code)** di setiap blok kode chat bubble untuk kemudahan menyalin kode.
   - Pengaturan interaktif untuk mengganti API Key, pilihan model Gemini, dan temperatur.
4. **Dukungan Mode Demo Cerdas**:
   - Jika belum memiliki Gemini API Key, agen tetap dapat dijalankan secara asinkron dengan simulasi penalaran: melakukan pencarian web DuckDuckGo asli, membaca/menulis berkas riil, dan membuka tab browser Threads Web Intent dengan draf postingan.

---

## Struktur Proyek

```text
my-ai-agent/
├── Cargo.toml            # Konfigurasi dependensi Rust & profil rilis
├── CHANGELOG.md          # Catatan versi rilisan aplikasi
├── BLUEPRINT.md          # Arsitektur sistem dan detail modul
├── src/
│   ├── main.rs           # Setup server Axum & router static/API
│   └── agent.rs          # Loop penalaran agen otonom, parser alat, & pencarian web
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

- **Mode Demo**: Cobalah mengirim pesan seperti `"Cari cara membuat routing di Axum Rust"`, `"Baca file Cargo.toml"`, atau `"Posting ke Threads tentang performa backend Rust"` untuk melihat agen memanggil perkakas backend secara dinamis.
- **Mode Gemini Riil**:
  1. Klik ikon **Pengaturan (Gir)** ⚙️ di pojok kiri bawah.
  2. Masukkan **Gemini API Key** Anda (didapatkan gratis dari Google AI Studio).
  3. Pilih model yang diinginkan (sangat direkomendasikan menggunakan `gemini-2.5-flash` atau `gemini-2.5-pro`).
  4. Klik **Simpan Perubahan**. Status badge di atas layar akan berubah menjadi **Terkoneksi**.
  5. Kirim pertanyaan kompleks dan perhatikan agen berpikir secara otonom!

---
*Dibuat dengan ❤️ menggunakan Rust & Gemini API.*
