/**
 * guides.js — Step-by-step forensic acquisition guides for CollectionLoom.
 *
 * Each guide follows ISO 27037 best practices and NIST SP 800-86 guidelines.
 * Used by GuideCard.svelte to render collapsible instruction panels.
 *
 * @typedef {Object} GuideStep
 * @property {string} title       — Short action heading
 * @property {string} description — Detailed instructions
 * @property {string} [warning]   — ⚠️ Caution (optional)
 *
 * @typedef {Object} Guide
 * @property {string}  title       — Panel heading
 * @property {string}  icon        — Emoji / icon identifier
 * @property {GuideStep[]} steps   — Ordered procedure
 * @property {string[]} references — Citations / resources
 */

/** @type {Guide} */
export const diskImagingGuide = {
  title: "Panduan Disk Imaging",
  icon: "●",   // disk
  steps: [
    {
      title: "Verifikasi write blocker",
      description:
        "Gunakan hardware write blocker bila memungkinkan (mis. Tableau, WiebeTech, dan sejenisnya). CollectionLoom akan mendeteksi blocker USB dan menampilkan badge hijau di titlebar. Jika tidak ada hardware blocker, pilih source disk lalu klik **Enable Software Write-Blocker**: Linux mengaktifkan BLKROSET read-only; macOS memaksa unmount volume sebelum imaging melalui `/dev/rdiskN`; Windows mengaktifkan mode read-only lewat IOCTL (butuh Administrator).",
      warning:
        "Software write-blocking memang mengurangi risiko, tetapi tidak menggantikan hardware bersertifikasi untuk barang bukti yang diperdebatkan. Jangan pernah me-mount volume tersangka dalam mode read-write sebelum imaging.",
    },
    {
      title: "Pilih sumber dan tujuan",
      description:
        "Identifikasi perangkat sumber dari daftar device. Pilih path tujuan pada volume penyimpanan bukti khusus yang punya ruang kosong cukup (ukuran source + 10% untuk overhead split/verifikasi). Format tujuan sebaiknya NTFS atau ext4, jangan FAT32 (batas file 4 GB).",
    },
    {
      title: "Atur opsi split dan verifikasi",
      description:
        "Untuk drive di atas 4 GB, set split size (misalnya 4096 MB) jika volume tujuan menggunakan FAT32 atau jika kamu ingin file pecahan yang lebih mudah dikelola. CollectionLoom mendukung hitungan byte u64, jadi tidak ada batas ukuran bawaan. Aktifkan verifikasi agar SHA-256 dihitung saat imaging dan dibandingkan setelah penulisan (untuk image satu part).",
    },
    {
      title: "Lakukan akuisisi image",
      description:
        "Mulai akuisisi. Tool akan membaca source device sektor demi sektor dan menulis format E01 (EnCase) atau dd / split-dd ke tujuan. Pantau progres real-time termasuk byte yang disalin, persen selesai, dan estimasi sisa waktu.",
    },
    {
      title: "Verifikasi hash dan dokumentasikan chain of custody",
      description:
        "Setelah akuisisi selesai, bandingkan hash (SHA-256) yang dihitung dari source dengan image hasil. Catat algoritma dan nilai hash di formulir chain-of-custody (CoC). Simpan image pada media write-once atau share NAS yang tahan manipulasi.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Guidelines for identification, collection, acquisition and preservation of digital evidence",
    "NIST SP 800-86 — Guide to Integrating Forensic Techniques into Incident Response",
    "NIST CFReDS — Computer Forensic Reference Data Sets (https://cfreds.nist.gov)",
  ],
};

/** @type {Guide} */
export const ramCaptureGuide = {
  title: "Panduan RAM Capture",
  icon: "◇",   // ram
  steps: [
    {
      title: "Periksa tool yang tersedia",
      description:
        "Pastikan tool akuisisi memori tersedia untuk target yang benar. Linux memakai AVML sebagai jalur utama dan LiME sebagai opsi lanjutan. Windows memakai WinPmem v4. macOS tidak menyediakan raw RAM acquisition di CollectionLoom; gunakan jalur volatile data alternatif.",
      warning:
        "Menjalankan binary yang tidak tepercaya pada mesin tersangka dapat mengubah bukti. Gunakan utility akuisisi yang tepercaya dan sudah di-hash dari media read-only.",
    },
    {
      title: "Tutup aplikasi yang tidak perlu",
      description:
        "Kurangi proses aktif pada target untuk menekan perubahan data volatil. Jangan mematikan atau me-reboot sistem karena data volatil akan hilang saat shutdown. Hindari operasi disk-intensive selama capture berlangsung.",
    },
    {
      title: "Ambil memori volatil",
      description:
        "Jalankan tool akuisisi dengan parameter yang sesuai. Untuk Linux: gunakan AVML sebagai jalur utama, atau LiME untuk kasus khusus. Untuk Windows: jalankan WinPmem v4 dan tentukan output path. Untuk macOS: gunakan Apple Volatile Data, bukan raw RAM dump.",
      warning:
        "Sebagian antivirus / EDR dapat menandai tool akuisisi memori sebagai berbahaya. Pre-authorise path tool atau jeda proteksi endpoint sementara jika itu aman dilakukan.",
    },
    {
      title: "Hitung dan catat hash",
      description:
        "Buat hash SHA-256 dari file hasil akuisisi segera setelah capture selesai. Catat hash bersama timestamp akuisisi, versi tool, dan hostname target di dokumentasi CoC. Simpan output pada media terenkripsi yang aksesnya terkontrol.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Digital evidence acquisition procedures",
    "NIST SP 800-86 — Live response and volatile data collection",
    "RFC 3227 — Guidelines for Evidence Collection and Archiving",
  ],
};

/** @type {Guide} */
export const appleVolatileDataGuide = {
  title: "Panduan Apple Volatile Data",
  icon: "",
  steps: [
    {
      title: "Pahami batasan platform",
      description:
        "CollectionLoom tidak menyediakan raw RAM dump universal untuk macOS, baik Intel maupun Apple Silicon. Fokuskan workflow pada alternatif volatile data seperti process list, network state, login/session artifacts, dan diagnostic output.",
    },
    {
      title: "Kumpulkan artefak volatil",
      description:
        "Ambil data proses aktif, koneksi jaringan, status login, autorun yang relevan, dan artifact triage yang tersedia pada versi macOS target. Simpan setiap output dengan timestamp dan source label.",
    },
    {
      title: "Tambahkan sistem diagnostik bila perlu",
      description:
        "Gunakan sysdiagnose atau live-response artifacts yang sesuai untuk membantu konteks investigasi. Ini bukan RAM acquisition, tetapi berguna untuk volatile-state preservation dan triage.",
    },
    {
      title: "Hash dan dokumentasikan",
      description:
        "Hash semua output dan catat sebagai volatile triage evidence, bukan memory dump. Simpan bersama chain of custody dan jelaskan bahwa sumbernya adalah alternatif volatile data, bukan raw RAM.",
    },
  ],
  references: [
    "Apple Platform Security / SIP and runtime protections",
    "ISO/IEC 27037:2012 — Volatile data handling",
    "NIST SP 800-86 — Live response considerations",
  ],
};

/** @type {Guide} */
export const mobileTriageGuide = {
  title: "Panduan Mobile Triage",
  icon: "☎",   // mobile
  steps: [
    {
      title: "Isolasi perangkat dari jaringan",
      description:
        "Masukkan perangkat ke Faraday bag atau aktifkan Airplane Mode segera untuk mencegah remote wipe, pesan masuk, atau cloud sync mengubah bukti. Jika perangkat terkunci, jangan mencoba menebak PIN karena bisa memicu wipe.",
      warning:
        "Perangkat yang tetap terhubung ke seluler/Wi‑Fi bisa di-wipe jarak jauh dalam hitungan detik. Gunakan wadah berpelindung untuk transport dan penyimpanan.",
    },
    {
      title: "Aktifkan USB debugging / developer mode",
      description:
        "Di Android: boot ke recovery mode atau gunakan MFD (Mobile Forensic Device) yang tepercaya untuk mengaktifkan ADB debugging tanpa menyentuh layar. Di iOS: masukkan perangkat ke DFU mode atau gunakan exploit bootloader yang kompatibel dengan checkpoint untuk logical acquisition. Dokumentasikan setiap interaksi.",
    },
    {
      title: "Ambil logical backup",
      description:
        "Jalankan tool akuisisi untuk membuat logical triage archive. Untuk Android, CollectionLoom mengutamakan shared storage capture sebagai arsip `.tar`/pull, bukan mengandalkan API ADB backup lama. Untuk ekstraksi lanjutan, gunakan tool seperti AFLogical OSE atau suite komersial. Ambil call log, SMS, kontak, aplikasi terpasang, dan file media sesuai izin dan kemampuan perangkat.",
    },
    {
      title: "Hash dan amankan backup",
      description:
        "Hitung SHA-256 untuk semua file yang diakuisisi dan file container (`.tar`, `.ab`, atau `.zip`). Catat hash di CoC. Simpan pada media terenkripsi. Jika perangkat mendukung file-level encryption, tangkap juga metadata enkripsinya untuk analisis lanjutan.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Mobile device acquisition considerations",
    "NIST SP 800-86 — Cell phone and PDA forensic procedures",
    "NIST IR 800-101 Rev 1 — Guidelines on Mobile Device Forensics",
  ],
};

/** @type {Guide} */
export const cloudEvidenceGuide = {
  title: "Panduan Cloud Evidence",
  icon: "☁",   // cloud
  steps: [
    {
      title: "Buat kredensial API sementara",
      description:
        "Masuk ke console IAM penyedia cloud dan buat pasangan API key yang dibatasi waktu dengan izin read-only. Atur masa berlaku sekecil mungkin (mis. 2 jam). Batasi policy hanya ke resource target yang diperlukan — jangan pernah memakai full-admin key.",
      warning:
        "API key dengan izin terlalu besar atau tanpa masa berlaku akan menciptakan risiko setelah tugas selesai. Selalu gunakan least privilege dan tetapkan timer pencabutan.",
    },
    {
      title: "Snapshot resource target",
      description:
        "Mulai snapshot disk virtual machine (EBS volume, Azure managed disk, GCP persistent disk) dan instance database. Tag setiap snapshot dengan case ID, timestamp, dan nama operator. Tunggu sampai status snapshot menjadi 'completed' sebelum lanjut.",
    },
    {
      title: "Unduh konfigurasi dan log",
      description:
        "Gunakan CLI atau API penyedia cloud untuk mengekspor data konfigurasi: policy IAM, network ACL, VPC flow logs, event CloudTrail / Activity Log, dan metadata instance. Simpan sebagai data terstruktur (JSON/CSV) dengan timestamp. Ambil minimal 90 hari log bila tersedia.",
    },
    {
      title: "Cabut kredensial sementara",
      description:
        "Segera cabut temporary API key setelah semua data dikumpulkan. Verifikasi bahwa key sudah nonaktif di IAM console. Catat aksi pencabutan di CoC dengan key ID dan timestamp pencabutan.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Remote / cloud evidence collection",
    "NIST SP 800-86 — Virtual and cloud forensic considerations",
    "CSA (Cloud Security Alliance) — Mappings to NIST SP 800-86",
  ],
};

/** @type {Guide} */
export const networkCaptureGuide = {
  title: "Panduan Network Capture",
  icon: "⊙",   // network
  steps: [
    {
      title: "Atur SPAN / port mirroring",
      description:
        "Jika capture pasif, atur SPAN atau port mirror pada managed switch untuk menduplikasi traffic dari VLAN/port target ke interface capture. Pastikan NIC capture berada dalam promiscuous mode. Untuk inline capture (honeypot), posisikan device capture di antara target dan gateway.",
      warning:
        "SPAN port yang tidak diprovisi dengan baik bisa menjatuhkan paket saat beban tinggi. Pastikan CPU switch dan storage capture mampu menahan bandwidth yang diperkirakan. Uji dengan pola traffic yang sudah dikenal sebelum mulai.",
    },
    {
      title: "Set filter capture (BPF)",
      description:
        "Definisikan Berkeley Packet Filter (BPF) untuk membatasi traffic yang ditangkap ke protokol relevan — misalnya `tcp`, `udp port 53`, atau `host 10.0.0.1`. Ini mengurangi noise dan kebutuhan storage. Gunakan capture length (snaplen) minimal 65535 byte agar paket tidak terpotong.",
    },
    {
      title: "Mulai packet capture",
      description:
        "Jalankan tcpdump, tshark, atau Wireshark dengan filter dan output file yang sudah diatur. Contoh tcpdump: `tcpdump -i eth0 -s 65535 -w evidence.pcap -C 1024 -W 10 'tcp port 80 or tcp port 443'`. Flag `-C` dan `-W` memutar file setiap 1024 MB dan menyimpan 10 file terakhir.",
    },
    {
      title: "Verifikasi dan hash file capture",
      description:
        "Setelah capture selesai, validasi file pcap dengan membukanya di Wireshark atau memakai `capinfos evidence.pcap`. Buat SHA-256 untuk setiap file pcap dan catat bersama timestamp mulai/selesai serta total packet count di CoC.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Network evidence acquisition",
    "NIST SP 800-86 — Network-based evidence",
    "NIST SP 800-94 — Guide to Intrusion Detection and Prevention Systems",
  ],
};

/** @type {Guide} */
export const writeBlockerGuide = {
  title: "Panduan Write Blocker",
  icon: "⚷",   // password/encrypt
  steps: [
    {
      title: "Periksa hardware write blocker",
      description:
        "Periksa secara visual perangkat write blocker untuk memastikan tidak ada kerusakan fisik. Pastikan interface yang benar (SATA, IDE, USB Bridge, NVMe) cocok dengan source drive. Pastikan firmware perangkat terbaru dan terdokumentasi. Hubungkan write blocker ke workstation akuisisi via USB atau eSATA.",
      warning:
        "Write blocker murah atau palsu bisa saja tidak benar-benar menegakkan read-only di level hardware. Gunakan hanya perangkat yang tercantum pada daftar tools yang disetujui NIST atau yang desain hardware-nya dipublikasikan.",
    },
    {
      title: "Hubungkan dan aktifkan perangkat",
      description:
        "Hubungkan source drive ke write blocker, lalu nyalakan blocker sebelum menyambungkannya ke workstation. Pastikan indikator LED menunjukkan 'Protected' / 'Read-Only'. CollectionLoom otomatis mendeteksi blocker USB Tableau/WiebeTech dan menampilkan badge hijau di titlebar. Jika tanpa hardware: klik **Enable Software Write-Blocker** — Linux: BLKROSET (`/sys/block/<dev>/ro` = 1); macOS: `diskutil unmountDisk force` lalu imaging via `/dev/rdiskN`; Windows: IOCTL read-only (jalankan sebagai Administrator).",
    },
    {
      title: "Verifikasi status read-only sebelum imaging",
      description:
        "Coba tulis marker uji ke perangkat: `dd if=/dev/zero of=/dev/sdX bs=512 count=1` seharusnya gagal dengan pesan 'Read-only file system' atau 'Permission denied'. Jika penulisan berhasil, segera batalkan dan ganti blocker. Jangan pernah imaging drive yang tersambung melalui blocker yang gagal.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Hardware write-blocking requirements",
    "NIST CFTT — Computer Forensics Tool Testing Program (https://www.nist.gov/itl/ssd/software-quality-group/computer-forensics-tool-testing-program-cftt)",
    "NIST SP 800-86 — Acquisition hardware requirements",
  ],
};

/** @type {Guide} */
export const acquireAllGuide = {
  title: "Panduan Acquire All",
  icon: "◉",
  steps: [
    {
      title: "Siapkan penyimpanan bukti",
      description:
        "Pilih folder output pada volume bukti khusus dengan ruang kosong minimal sebesar ukuran source drive ditambah overhead 10%. Gunakan NTFS, APFS, atau ext4 — hindari FAT32 untuk image di atas 4 GB kecuali split diaktifkan (mis. 4096 MB). CollectionLoom melakukan streaming sektor demi sektor tanpa batas ukuran di level aplikasi.",
    },
    {
      title: "Deteksi dan pilih modul",
      description:
        "Klik **Detect Sources** untuk menyegarkan daftar disk, tool RAM, interface network, dan sumber mobile (bukan folder output). Aktifkan hanya modul yang dibutuhkan (Disk, RAM, Network, Mobile). Setiap modul berjalan berurutan; kegagalan satu modul tidak menghentikan modul lain.",
    },
    {
      title: "Aktifkan write protection sebelum disk imaging",
      description:
        "Saat modul Disk aktif, pilih source device. CollectionLoom memeriksa hardware blocker secara otomatis (badge hijau di titlebar). Jika tidak aktif, klik **Enable Software Write-Blocker** sebelum mulai — Linux BLKROSET, macOS unmount + raw disk path, Windows read-only IOCTL. Acquire All akan mengaktifkan software blocking otomatis saat disk imaging dimulai bila proteksi belum aktif.",
      warning:
        "Jangan mulai disk acquisition pada volume yang ter-mount read-write. Software blocking di Windows membutuhkan hak Administrator.",
    },
    {
      title: "Atur split untuk drive besar",
      description:
        "Untuk drive di atas 4 GB (atau source multi-terabyte), set **Split (MB)** ke 4096 atau lebih tinggi agar tiap segmen tetap dalam batas filesystem dan lebih mudah disalin. Biarkan 0 untuk image kontigu tunggal jika tujuan mendukung file besar. Progres akan menampilkan kapasitas yang mudah dibaca (TB/GB) saat imaging.",
    },
    {
      title: "Jalankan akuisisi batch dan verifikasi",
      description:
        "Klik **Start Acquire All**. Modul berjalan mengikuti urutan volatilitas (RFC 3227 / NIST SP 800-86): **Network → RAM → Mobile → Disk → Cloud**. Source volatil diambil lebih dulu; disk imaging dijalankan terakhir bila diaktifkan. Setiap output volatil di-hash SHA-256 dengan verifikasi dual-read. Setelah selesai, catat hash di tab Chain of Custody dan ekspor case bundle saat siap.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Integrated digital evidence collection",
    "NIST SP 800-86 — Live response and volatile data ordering",
    "NIST CFReDS — https://cfreds.nist.gov",
  ],
};

/** @type {Guide} */
export const snapshotGuide = {
  title: "Panduan Snapshot",
  icon: "◈",   // snapshot
  steps: [
    {
      title: "Ambil baseline snapshot sistem bersih",
      description:
        "Sebelum menjalankan tool yang berpotensi volatil, tangkap baseline sistem yang sedang berjalan: daftar proses aktif (`ps aux`), koneksi network aktif (`ss -tulpn`), kernel module yang ter-load (`lsmod`), dan file handle terbuka (`lsof`). Simpan output ke direktori bertanggal di bawah folder bukti case.",
    },
    {
      title: "Jalankan aksi atau tool target",
      description:
        "Jalankan tool forensik atau aplikasi yang sedang diuji. Catat command yang dipakai, start time (UTC), dan parameter apa pun yang digunakan. Jika tool mengubah state sistem (memuat kernel module, menulis log), catat sebagai efek samping.",
    },
    {
      title: "Ambil snapshot setelah eksekusi",
      description:
        "Segera setelah tool selesai, jalankan ulang perintah pengumpulan informasi yang sama seperti pada langkah 1. Ambil semua output baru — jangan pakai ulang file baseline. Catat end time (UTC).",
    },
    {
      title: "Analisis perbedaannya",
      description:
        "Bandingkan baseline dan snapshot setelah eksekusi: proses, koneksi network, module yang ter-load, dan file handle. Identifikasi proses baru atau yang berhenti, port yang terbuka, kernel module yang ter-load, dan penulisan ke filesystem. Dokumentasikan semua perubahan di laporan case.",
    },
  ],
  references: [
    "NIST SP 800-86 — Live forensic data collection and state change analysis",
    "ISO/IEC 27037:2012 — State capture and documentation",
    "SANS — Forensic Analysis of System State Snapshots",
  ],
};

/** @type {Guide} */
export const verificationGuide = {
  title: "Panduan Verifikasi Bukti",
  icon: "✓",   // verify
  steps: [
    {
      title: "Pilih algoritma verifikasi",
      description:
        "Pilih algoritma hash kriptografis untuk verifikasi integritas. SHA-256 adalah minimum yang direkomendasikan (NIST SP 800-131A). Hindari MD5 dan SHA-1 untuk kasus baru kecuali benar-benar diperlukan untuk interoperabilitas legacy. Algoritma yang sama harus dipakai untuk hash source dan image.",
    },
    {
      title: "Masukkan nilai hash yang diharapkan",
      description:
        "Jika expected hash sudah dicatat saat akuisisi (mis. di formulir CoC atau file hashset bertanda tangan), masukkan ke field 'Expected Hash'. Nilai ini akan dibandingkan dengan hash image hasil akuisisi untuk memastikan integritas.",
    },
    {
      title: "Hitung hash file bukti",
      description:
        "Pilih file bukti atau device lalu jalankan verifikasi. Tool akan menghitung hash item yang dipilih dan membandingkannya dengan nilai yang diharapkan. Jika cocok, integritas terkonfirmasi; jika tidak cocok, itu menunjukkan manipulasi, korupsi, atau kesalahan penyalinan dan bukti menjadi tidak valid.",
      warning:
        "Hash mismatch TIDAK otomatis berarti manipulasi sengaja — bisa juga disebabkan salinan yang tidak lengkap, error disk, atau bit-rot pada media penyimpanan. Selidiki penyebabnya sebelum menyimpulkan integritas bukti terganggu.",
    },
    {
      title: "Dokumentasikan hasil verifikasi",
      description:
        "Catat hasil verifikasi (passed/failed), hash yang dihitung, expected hash, algoritma, timestamp, dan nama operator di CoC. Hasil yang lulus juga bisa dicetak dan ditandatangani untuk dokumentasi chain-of-custody fisik.",
    },
  ],
  references: [
    "NIST SP 800-86 — Hash verification for forensic integrity",
    "NIST SP 800-131A — Transitioning cryptographic algorithms",
    "ISO/IEC 27037:2012 — Integrity verification requirements",
  ],
};

/** @type {Guide} */
export const encryptionGuide = {
  title: "Panduan Asesmen Enkripsi",
  icon: "⚷",   // encryption
  steps: [
    {
      title: "Pindai volume/container terenkripsi pada target",
      description:
        "Gunakan tool deteksi untuk memindai sistem target terhadap volume, container, dan file terenkripsi. Tool akan memeriksa penanda enkripsi umum: header TrueCrypt/VeraCrypt, partisi LUKS, volume BitLocker, container FileVault, dan ZIP terenkripsi. Tinjau laporan hasil scan.",
    },
    {
      title: "Tinjau temuan dan identifikasi jenis enkripsi",
      description:
        "Untuk setiap container enkripsi yang terdeteksi, catat jenis enkripsi, algoritma (AES-256, Twofish, Serpent, dll.), dan apakah material kunci (recovery key, key file) tersimpan lokal. Untuk BitLocker di lingkungan yang terhubung ke AD, cek apakah recovery key disimpan di Active Directory.",
    },
    {
      title: "Tindak lanjuti rekomendasi",
      description:
        "Berdasarkan scan, lakukan aksi yang direkomendasikan: (a) jika recovery key tersedia, decrypt volume dan ambil plaintext; (b) jika volume sedang terbuka oleh user yang login, lakukan live acquisition sebelum shutdown; (c) jika tidak ada kunci yang bisa didapat, dokumentasikan enkripsi sebagai hambatan dan segel perangkat untuk imaging forensik dengan catatan status enkripsi.",
      warning:
        "Jangan pernah mencoba brute-force atau dictionary attack pada volume terenkripsi kecuali secara eksplisit diizinkan oleh otoritas penyidik. Tindakan itu dapat mengubah metadata volume dan merusak bukti percobaan dekripsi.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Encrypted evidence handling",
    "NIST SP 800-86 — Encryption and key recovery considerations",
    "NIST SP 800-88 Rev 1 — Guidelines for Media Sanitization (encryption disposal context)",
  ],
};
