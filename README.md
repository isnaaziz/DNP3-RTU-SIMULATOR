# Simulator RTU DNP3

Simulator RTU (outstation) DNP3 dengan web UI. Dibuat dengan Rust dan library `dnp3` dari Step Function I/O, yaitu library yang sama dengan yang dipakai `iconics-dnp3-outstation`.

## Fitur

- Tambah, ubah, dan hapus point dari UI: Binary Input, Double-bit Input, Binary Output (kontrol), Analog Input, Analog Output (setpoint), dan Counter.
- Index (IOA) setiap point bisa diatur dan diubah kapan saja; perubahan langsung terlihat oleh master tanpa restart.
- Ubah nilai manual: toggle BI, pilih posisi DBI (INTER/OFF/ON/BAD), set nilai analog dan counter.
- Quality per point: ONLINE, RESTART, COMM_LOST, REMOTE_FORCED, LOCAL_FORCED (klik chip untuk menyalakan/mematikan).
- Simulasi otomatis: random walk, sinus, ramp, toggle; counter bertambah otomatis.
- Kontrol asli dari master:
  - CROB (g12v1) ke Binary Output, mendukung SELECT-BEFORE-OPERATE dan DIRECT-OPERATE, latch maupun pulse, trip/close.
  - Kontrol 1 index (arah dari kode TRIP/CLOSE atau LATCH ON/OFF) atau 2 index (BO terpisah untuk Open dan Close; perintah apa pun ke index Open membuka, ke index Close menutup).
  - Analog output (g41v1–v4) ke Analog Output.
  - Feedback ke point status: BO → DBI melewati posisi intermediate selama waktu tempuh, lalu ke OFF/ON; BO → BI; AO → AI.
  - Status Local/Remote per kontrol: point BI atau DBI dengan index sendiri (BI 1 = Local, 0 = Remote; DBI 2 = Local, 1 = Remote) yang ikut dikirim ke master. Saat Local, perintah ditolak dengan status `Local`.
  - Index yang tidak ada dijawab `NotSupported`.
- Unsolicited, time sync dari master, dan log semua kontrol serta koneksi master.
- Semua konfigurasi dilakukan dari UI dan tersimpan otomatis.
- Export konfigurasi ke file JSON dan import kembali, misalnya untuk memindahkan daftar point ke simulator lain.

## Menjalankan

```bash
cargo run --release -- --http 0.0.0.0:8080
```

Buka `http://localhost:8080`. Saat pertama kali dijalankan, simulator berisi contoh point (CB dengan kontrol, LBS, tegangan, arus, frekuensi, kWh, setpoint).

| Argumen | Default | Keterangan |
|---|---|---|
| `--http` | `0.0.0.0:8080` | Alamat web UI |
| `--data` | `simulator-data.json` | File penyimpanan internal |

File `--data` hanya tempat simulator menyimpan keadaannya; tidak perlu diedit manual. Semua perubahan dilakukan dari UI.

## Tampilan

UI terdiri dari tiga kolom yang diperbarui setiap 0,5 detik:

- **Circuit Breakers**: satu kartu per point kontrol (BO). Lampu hijau = OPEN, merah = CLOSE, keduanya berkedip saat intermediate. Tombol Open/Close/Inter 00/Bad 11 mengubah status lapangan secara langsung (tanpa lewat master). Saklar L/R mengubah point status L/R (index-nya tampil di kartu) sehingga master menerima event perubahan; saat Local, kontrol dari master ditolak. Kontrol tanpa point L/R memakai mode L/R internal yang tidak dikirim ke master.
- **Telesignals**: BI dan DBI yang tidak dipakai sebagai status CB. Tombol `A` menyalakan/mematikan simulasi otomatis.
- **Telemetries**: AI, setpoint (AO), dan counter dengan satuan. Klik nilai untuk mengetik nilai baru, tombol +/− untuk menaikkan/menurunkan, `A` untuk simulasi otomatis.

Angka bergaris putus-putus (index status, index kontrol Open/Close, waktu tempuh, index L/R, dan index di Telesignals/Telemetries) bisa diklik untuk diubah langsung; tekan Enter untuk menyimpan atau Esc untuk batal. Pilihan **1 index / 2 index** di kartu mengubah tipe kontrol. Klik nama point atau ikon pensil untuk mengubah semua pengaturannya (tipe, index, nama, satuan, quality, simulasi, feedback kontrol, L/R). Saat menambah kontrol CB dengan status feedback yang belum ada, point status ikut dibuat otomatis.

## Export dan import konfigurasi

Panel **Konfigurasi** di UI:

- **Export** mengunduh `simulator-rtu-dnp3.json` (format rapi, satu field per baris) berisi pengaturan outstation dan semua point (tipe, index, nama, nilai, quality, class, simulasi, feedback kontrol, mode LOCAL).
- **Template** mengunduh `template-import-simulator-rtu-dnp3.json`: contoh lengkap setiap tipe point (CB double, LBS single, telesignal, telemetry, counter, setpoint) beserta bagian `_keterangan` yang menjelaskan setiap kolom. Edit template ini lalu import; bagian `_keterangan` diabaikan saat import. File yang sama ada di [`static/import-template.json`](static/import-template.json).
- **Import** mengganti seluruh pengaturan dan point dengan isi file, lalu me-restart outstation. Isi file divalidasi dulu (pengaturan, nilai per tipe, index ganda); jika ada yang salah, import dibatalkan dan konfigurasi lama tetap utuh.

Format file export:

```json
{
  "settings": { "bind_ip": "0.0.0.0", "port": 20000, "outstation_address": 10, "master_address": 1, "event_buffer": 200, "unsolicited": true },
  "points": [
    { "kind": "double_bit", "index": 0, "name": "CB Feeder 1", "value": 2, "flags": 1, "class": 1, "sim": null, "feedback": null, "local": false },
    { "kind": "binary_output", "index": 0, "name": "CB Feeder 1 Control", "value": 1, "flags": 1, "class": 1, "sim": null,
      "feedback": { "kind": "double_bit", "index": 0, "travel_ms": 2000 }, "local": false }
  ]
}
```

Nilai `kind`: `binary_input`, `double_bit`, `binary_output`, `counter`, `analog_input`, `analog_output`. Kolom `unit`, `flags`, `class`, `sim`, `feedback`, `lr`, `close_index`, dan `local` boleh dihilangkan (default: tanpa satuan, ONLINE, Class 1, tanpa simulasi, tanpa feedback).

Pengaturan DNP3 (IP bind, port, alamat outstation, alamat master, event buffer, unsolicited) diatur dari panel **Pengaturan outstation** di UI. Default: `0.0.0.0:20000`, outstation `10`, master `1`.

## Menghubungkan master

| Setting di master | Nilai default |
|---|---|
| IP / Port | IP mesin simulator / `20000` |
| Master address | `1` |
| Outstation address | `10` |

Semua point memakai variasi berikut:

| Tipe | Statis | Event |
|---|---|---|
| Binary Input | g1v2 | g2v2 (dengan waktu) |
| Double-bit Input | g3v2 | g4v2 (dengan waktu) |
| Binary Output Status | g10v2 | g11v2 (dengan waktu) |
| Counter | g20v1 | g22v5 (dengan waktu) |
| Analog Input | g30v5 (float) | g32v7 (float dengan waktu) |
| Analog Output Status | g40v3 (float) | g42v7 (float dengan waktu) |

## Contoh skenario kontrol CB

1. Point `BO 0 "CB Feeder 1 Control"` punya feedback ke `DBI 0` dengan waktu tempuh 2000 ms.
2. Master mengirim CROB TRIP (atau LATCH_OFF) ke BO 0.
3. Simulator menjawab `Success`, DBI 0 langsung menjadi INTERMEDIATE (0), lalu setelah 2 detik menjadi OFF (1). Kedua perubahan dikirim sebagai event bertimestamp.
4. Centang **Mode LOCAL** di BO 0 untuk menguji penolakan kontrol oleh RTU.
