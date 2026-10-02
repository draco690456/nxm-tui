# R5 — OS-independent keystore (study)

- **Label:** `wayfinder:research`
- **Type:** Research — study before deciding/implementing
- **Status:** ✅ Resolved 2026-10-02 — studio completo in `nxm-docs/research/2026-10-02_os-independent-keystore-R5.md`
- **Related:** [D7-key-resolution-config.md](D7-key-resolution-config.md),
  [R1-keyring-crate.md](R1-keyring-crate.md),
  [MAP-provider-keys-models.md](../MAP-provider-keys-models.md)

## Motivazione

Oggi la risoluzione key dipende dal keychain **nativo del sistema operativo**
(`keyring 4` → `apple-native-keyring-store` su macOS, servizi diversi su
Linux/Windows). Due problemi emersi:

1. **Dipendenza dall'OS**: comportamento, disponibilità e prompt variano per
   piattaforma; headless/CI/container spesso non hanno un keychain. Serve uno
   storage **portabile e deterministico**, indipendente dal backend OS.
2. **Non testabile** (scoperto 2026-10-02): la prod usa `keyring::Entry` (store
   nativo), i test montano un mock su `keyring_core` → namespace scollegati, il
   mock è invisibile a `keyring`. Il test `keychain_beats_env` è perciò
   `#[ignore]`. Manca un **seam `KeyStore` iniettabile**.

Mitigazione già in essere (non sostituisce lo studio): modalità **env-first**
con `.env` locale (`dotenvy`) per lo sviluppo — vedi CHANGELOG 2026-10-02.

## Domande da rispondere (study)

1. **Trait `KeyStore` interno** — definire un seam
   (`get/set/delete(provider) -> Result<...>`) con impl multiple dietro, così
   `resolve_key` non dipende da un backend concreto ed è testabile con un fake.
2. **Backend portabile indipendente dall'OS** — opzioni da valutare:
   - file cifrato locale (es. AES-GCM con chiave derivata da passphrase/argon2),
     path XDG/`dirs`; pro/contro vs. keychain nativo.
   - `keyring 4` con *secure-enclave/file store* esplicito e uniforme su tutte
     le piattaforme (esiste un backend file portabile in keyring-rs?).
   - solo-env + `.env` (già fatto) come livello 0; quando basta?
3. **Sicurezza**: a riposo (cifratura, permessi file 0600), in memoria
   (zeroize del buffer), mai in log/argv/history (regola D7 mai-log).
4. **Compat con D7**: ordine di risoluzione, migrazione delle entry esistenti
   nel keychain nativo verso il nuovo store (se si cambia backend).
5. **Headless/CI**: comportamento deterministico senza alcun keychain.
6. **Dipendenze**: pin esatti, licenze compatibili MIT; evitare crate pesanti.

## Done when

Documento `research/os-independent-keystore.md` con: confronto backend
(tabella pro/contro + sicurezza + portabilità + testabilità), raccomandazione
di un backend + del seam `KeyStore`, e uno spec implementabile (firme, ordine,
migrazione) pronto per un ticket di implementazione. Nessun codice qui.
