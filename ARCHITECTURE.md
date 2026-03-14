# ROOT v2.0 — Архитектура и взаимодействие модулей

## Структура проекта

```
rust/src/
├── lib.rs                        ← точка входа библиотеки (cdylib для Flutter)
├── identity/                     ← криптографическая идентичность
│   ├── mod.rs                    ← реэкспорт
│   ├── seed.rs                   ← SecretSeed
│   ├── keys.rs                   ← Identity (Ed25519)
│   ├── shamir.rs                 ← ShamirVault (3/5)
│   └── protected.rs              ← ProtectedKey (XOR маска)
├── storage/                      ← база данных
│   ├── mod.rs                    ← реэкспорт
│   ├── constants.rs              ← KEY_LEN, Argon2 параметры
│   ├── error.rs                  ← StorageError
│   ├── key.rs                    ← StorageKey (Argon2id)
│   ├── models.rs                 ← Message, Contact
│   ├── merkle.rs                 ← MerkleTree
│   ├── panic.rs                  ← PanicButton
│   └── database.rs               ← Database (SQLite движок)
├── network/                      ← P2P сеть
│   ├── mod.rs                    ← реэкспорт
│   ├── behaviour.rs              ← RootBehaviour (Gossipsub + mDNS)
│   ├── channels.rs               ← start_node_channels (Flutter)
│   └── node.rs                   ← start_node (CLI)
├── economy/                      ← токен SAP
│   ├── mod.rs                    ← реэкспорт
│   ├── constants.rs              ← Hard Cap, Velocity, Vesting...
│   ├── types.rs                  ← EconomyError, Transaction, TxType
│   ├── vesting.rs                ← VestingSchedule
│   ├── protection.rs             ← VelocityTracker, AnomalyDetector, PersonhoodRegistry
│   ├── account.rs                ← Account
│   ├── treasury.rs               ← Treasury + стабфонд
│   ├── consensus.rs              ← WitnessConfig, Proof-of-Relay
│   └── ledger.rs                 ← Ledger (главный движок)
├── api/                          ← FFI интерфейс для Flutter
│   ├── mod.rs                    ← RootApi struct + все impl
│   ├── types.rs                  ← IdentityInfo, MessageInfo, BalanceInfo...
│   ├── state.rs                  ← глобальное состояние (lazy_static)
│   ├── identity.rs               ← generate_identity, restore_identity
│   ├── database.rs               ← unlock_database, panic_button
│   ├── messaging.rs              ← send_message, get_messages
│   ├── contacts.rs               ← add_contact, get_contacts
│   ├── economy.rs                ← get_balance, transfer, stake
│   ├── p2p.rs                    ← start_p2p_node, send_p2p_message
│   └── utils.rs                  ← get_version, validate_public_key
└── cli/                          ← терминальный интерфейс
    ├── main.rs                   ← точка входа бинарника
    └── commands/
        ├── mod.rs
        ├── identity.rs           ← generate, restore, show
        ├── node.rs               ← listen, connect
        ├── server.rs             ← bootstrap сервер (VPS)
        ├── contacts.rs           ← add, list
        ├── messages.rs           ← list, send
        └── db.rs                 ← unlock, verify
```

---

## Как модули взаимодействуют

```
Flutter (Dart)
    │
    │  flutter_rust_bridge (FFI)
    ▼
api/mod.rs  ←──── RootApi::unlock_database()
    │               RootApi::generate_identity()
    │               RootApi::start_p2p_node()
    │               RootApi::send_message()
    │               RootApi::get_balance()
    │               ...
    │
    ├──▶ api/state.rs        CURRENT_IDENTITY, CURRENT_DB,
    │                        CURRENT_LEDGER, P2P_SENDER,
    │                        INCOMING_QUEUE, PEER_COUNT
    │
    ├──▶ api/identity.rs ──▶ identity/keys.rs    (Identity::generate)
    │                        identity/seed.rs     (SecretSeed)
    │
    ├──▶ api/database.rs ──▶ storage/database.rs (Database::open)
    │                        storage/key.rs       (StorageKey::from_password)
    │                        storage/panic.rs     (PanicButton::activate)
    │
    ├──▶ api/messaging.rs──▶ storage/database.rs (save_message, get_messages)
    │                        storage/models.rs    (Message::new)
    │
    ├──▶ api/contacts.rs ──▶ storage/database.rs (add_contact, get_contacts)
    │                        api/utils.rs         (validate_public_key)
    │
    ├──▶ api/economy.rs  ──▶ economy/ledger.rs   (transfer, p2p_exchange)
    │                        economy/account.rs   (Account)
    │                        economy/vesting.rs   (VestingSchedule)
    │                        economy/treasury.rs  (Treasury)
    │
    ├──▶ api/p2p.rs      ──▶ network/channels.rs (start_node_channels)
    │                        api/state.rs         (P2P_SENDER, INCOMING_QUEUE)
    │
    └──▶ api/utils.rs        get_version, validate_public_key


CLI (root-cli бинарник)
    │
    ├──▶ cli/commands/identity.rs ──▶ identity/keys.rs
    ├──▶ cli/commands/node.rs     ──▶ network/node.rs (start_node)
    ├──▶ cli/commands/server.rs   ──▶ network/behaviour.rs (RootBehaviour)
    ├──▶ cli/commands/contacts.rs ──▶ storage/database.rs
    ├──▶ cli/commands/messages.rs ──▶ storage/database.rs
    └──▶ cli/commands/db.rs       ──▶ storage/database.rs
```

---

## Поток данных: запуск Flutter приложения

```
1. main.dart
   └─▶ RootApi.unlockDatabase(password, path)
         └─▶ StorageKey::from_password()  ← Argon2id 64MB/3iter (~300ms)
         └─▶ Database::open()             ← SQLite открывается
         └─▶ Database::initialize()       ← CREATE TABLE IF NOT EXISTS
         └─▶ db.load_identity()           ← если есть — загружаем Identity

2. main.dart
   └─▶ RootApi.generateIdentity()         ← только при первом запуске
         └─▶ Identity::generate()         ← OsRng → 32 байта → Mnemonic → Ed25519
         └─▶ db.save_identity()           ← мнемоника в SQLite
         └─▶ Ledger::new()                ← инициализация экономики

3. main.dart
   └─▶ RootApi.startP2pNode()
         └─▶ identity.signing_key_bytes() ← стабильный Ed25519 ключ
         └─▶ start_node_channels(key)     ← запуск libp2p Swarm
               └─▶ Gossipsub подписка на "root-network-v2"
               └─▶ mDNS для поиска в локальной сети
               └─▶ фоновый поток: P2P → INCOMING_QUEUE

4. Flutter polling (каждые 2 сек)
   └─▶ RootApi.getIncomingMessages()
         └─▶ INCOMING_QUEUE.drain()       ← забираем и очищаем очередь
```

---

## Поток данных: отправка сообщения

```
Пользователь вводит текст в Flutter UI
    │
    ▼
RootApi.sendMessage(to_key, content)
    │
    ├─▶ storage/database.rs::save_message()
    │       └─▶ INSERT INTO messages
    │       └─▶ MerkleTree::add_leaf(hash)
    │       └─▶ INSERT INTO merkle_roots
    │
    └─▶ RootApi.sendP2pMessage(content)
            └─▶ P2P_SENDER.try_send(content)
                    └─▶ network/channels.rs
                            └─▶ gossipsub.publish("root-network-v2", bytes)
```

---

## Поток данных: получение сообщения

```
libp2p Gossipsub получает пакет из сети
    │
    ▼
network/channels.rs  (фоновый tokio::spawn)
    └─▶ gossipsub::Event::Message { propagation_source, message }
            └─▶ P2pMessage { from_peer, content, timestamp }
                    └─▶ INCOMING_QUEUE.push(msg)

Flutter polling (каждые 2 секунды)
    └─▶ RootApi.getIncomingMessages()
            └─▶ INCOMING_QUEUE.drain()
                    └─▶ [MessageInfo, MessageInfo, ...]  → UI
```

---

## Поток данных: экономика SAP

```
RootApi.transfer(to_key, amount_sap)
    │
    ▼
api/economy.rs
    └─▶ CURRENT_LEDGER (Mutex<Option<Ledger>>)
            └─▶ economy/ledger.rs::transfer()
                    ├─▶ account.anomaly.check_frozen()    ← заморожен?
                    ├─▶ account.check_rate_limit()        ← >10 tx/сек?
                    ├─▶ баланс >= amount + fee?
                    ├─▶ from.balance -= amount + fee
                    ├─▶ to.balance   += amount
                    ├─▶ treasury.deposit(fee)             ← 0.1% → казна
                    └─▶ Transaction::new()                ← SHA256 ID

RootApi.p2pExchange(to_key, amount_sap)
    └─▶ economy/ledger.rs::p2p_exchange()
            ├─▶ reputation >= 70?                        ← Механизм 5
            ├─▶ velocity.check_and_record()              ← Механизм 1: 100 SAP/день
            ├─▶ vesting.spend()                          ← Механизм 2: Genesis lock
            ├─▶ burn 1% навсегда                         ← Механизм 3
            ├─▶ treasury.deposit(fee)
            └─▶ anomaly.record_sale()                    ← Механизм 6: детектор
```

---

## Модуль identity — детали

| Файл | Что делает | Ключевые типы |
|------|-----------|---------------|
| `seed.rs` | Обёртка над [u8;64] с zeroize | `SecretSeed` |
| `keys.rs` | Ed25519 генерация из BIP39 мнемоники | `Identity` |
| `shamir.rs` | Разделение ключа 3/5 (threshold secret sharing) | `ShamirVault`, `ShamirError` |
| `protected.rs` | XOR маскировка ключа в памяти при фоне | `ProtectedKey` |

**Алгоритм генерации ключа:**
```
OsRng (32 байта) → BIP39 Mnemonic (24 слова)
                              ↓
                   mnemonic.to_seed("ROOT_v2")  ← PBKDF2
                              ↓
                   seed[0..32] → Ed25519 SigningKey
                              ↓
                   SigningKey  → VerifyingKey (публичный ключ)
```

---

## Модуль storage — детали

| Файл | Что делает | Ключевые типы |
|------|-----------|---------------|
| `constants.rs` | KEY_LEN=32, Argon2 64MB/3iter/1par | — |
| `error.rs` | Все ошибки хранилища | `StorageError` |
| `key.rs` | Argon2id: пароль → 32-байтный ключ | `StorageKey` |
| `models.rs` | Message (id, from, to, content, ts) | `Message`, `Contact` |
| `merkle.rs` | SHA256 дерево для верификации | `MerkleTree` |
| `panic.rs` | Уничтожение ключа при принуждении | `PanicButton` |
| `database.rs` | SQLite: CRUD сообщений, контактов | `Database` |

**Защита данных:**
```
Пароль пользователя
    ↓  Argon2id (64MB RAM, 3 итерации, ~300ms)
StorageKey [u8;32]   ← никогда не на диск, zeroize при drop
    ↓  PRAGMA key (SQLCipher AES-256)
SQLite файл (зашифрован)
```

---

## Модуль network — детали

| Файл | Что делает | Используется |
|------|-----------|-------------|
| `behaviour.rs` | RootBehaviour (Gossipsub + mDNS), build_gossipsub() | channels.rs, node.rs, cli/server.rs |
| `channels.rs` | P2P узел с mpsc каналами для Flutter | api/p2p.rs |
| `node.rs` | Интерактивный P2P узел для CLI | cli/commands/node.rs |

**Топик:** `root-network-v2`
**PeerID:** стабильный — derived из Ed25519 signing_key пользователя

---

## Модуль economy — детали

| Файл | Что делает |
|------|-----------|
| `constants.rs` | Hard Cap 1B SAP, DROPS_PER_SAP=100M, лимиты |
| `types.rs` | Transaction (SHA256 ID), TxType, EconomyError |
| `vesting.rs` | Genesis бонус: 10% сразу, 100% за 365 дней |
| `protection.rs` | VelocityTracker (100 SAP/день), AnomalyDetector, PersonhoodRegistry |
| `account.rs` | Баланс, stake, репутация, история tx |
| `treasury.rs` | Казначейство: комиссии, slash, стабфонд 20% |
| `consensus.rs` | WitnessConfig: 1/3/5/7 свидетелей по сумме |
| `ledger.rs` | transfer, p2p_exchange, stake, slash, genesis, relay |

**6 механизмов защиты от dump:**
1. **Velocity Limit** — max 100 SAP/день на продажу
2. **Vesting** — Genesis бонус разблокируется 365 дней
3. **Burn 1%** — при каждой P2P сделке сжигается навсегда
4. **Стабфонд** — 20% Treasury на выкуп при падении курса -30%
5. **Proof of Personhood** — 1 устройство = 1 Genesis бонус
6. **Anomaly Detector** — заморозка 72ч при продаже >50% баланса

---

## Модуль api — детали

| Файл | FFI функции |
|------|-------------|
| `state.rs` | CURRENT_IDENTITY, CURRENT_DB, CURRENT_LEDGER, P2P_SENDER, INCOMING_QUEUE, PEER_COUNT |
| `identity.rs` | `generate_identity()`, `restore_identity()`, `get_public_key()`, `sign_message()` |
| `database.rs` | `unlock_database()`, `panic_button()`, `verify_db_integrity()`, `is_panic_activated()` |
| `messaging.rs` | `send_message()`, `get_messages()`, `get_unread_count()`, `mark_message_read()` |
| `contacts.rs` | `add_contact()`, `get_contacts()` |
| `economy.rs` | `get_balance()`, `transfer()`, `p2p_exchange()`, `get_vesting_info()`, `stake_node()`, `unstake_node()`, `get_node_status()`, `claim_genesis()` |
| `p2p.rs` | `start_p2p_node()`, `send_p2p_message()`, `is_p2p_running()`, `get_incoming_messages()`, `get_peer_count()` |
| `utils.rs` | `get_version()`, `validate_public_key()` |

---

## CLI интерфейс

```bash
# Пользовательский режим
root-cli identity generate
root-cli identity restore "слово1 слово2 ... слово24"
root-cli db unlock --password <пароль> --path root.db
root-cli contacts add <64-символа-hex> "Имя"
root-cli contacts list
root-cli messages list
root-cli messages send <ключ> "текст"
root-cli node listen
root-cli node connect /ip4/1.2.3.4/tcp/7001/p2p/<PeerID>

# Сервер (VPS / bootstrap узел)
root-cli server start --port 7001
root-cli server status
```

---

## Правила для cargo build

```bash
# Windows — обычная сборка
cd D:\work_rust\root_app\rust
cargo check
cargo build --release

# Android NDK
$env:ANDROID_NDK_HOME = "C:\Users\Filip\AppData\Local\Android\Sdk\ndk\28.2.13676358"
cargo ndk -t arm64-v8a --platform 24 build --release --lib
copy target\aarch64-linux-android\release\libroot_core.so ..\android\app\src\main\jniLibs\arm64-v8a\

# CLI бинарник (для VPS)
cargo build --release --bin root-cli
```

---

## Что делать ПОСЛЕ этого рефакторинга

Следующие задачи в порядке приоритета:

1. **Bootstrap сервер** — запустить `root-cli server start --port 7001` на VPS, добавить его адрес в `network/channels.rs` как fallback если mDNS не работает

2. **E2E шифрование** — конвертировать Ed25519 → X25519, шифровать `content` в `Message` перед сохранением в БД

3. **Store-and-forward** — relay узлы со stake хранят сообщения офлайн пользователей до 7 дней

4. **Реальный пароль** — убрать хардкоженные пароли из тестов, добавить экран ввода пароля во Flutter

5. **QR коды** — `qr_flutter` + `mobile_scanner` для обмена публичными ключами
