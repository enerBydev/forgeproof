# Forgeproof

Especialistas de ingeniería versionados, paquetes firmados y evidencia comprobable para asistentes de programación.

La primera entrega implementa **`engctl verify`**, un verificador local en Rust, y un **inspector de recibos para Vercel**. Comprueba firma Minisign, contenido exacto, cliente declarado y vigencia de revocaciones. La incertidumbre no permite aprobar un paquete.

Esta entrega todavía no instala agentes ni ofrece un servidor MCP. La compatibilidad con Codex, Claude Code, ChatGPT y Claude requiere adaptadores y pruebas reales en entregas posteriores; un nombre de cliente en el manifiesto no la certifica.

## Empezar

En Linux x86-64 con Nix y flakes habilitados:

```sh
git clone https://github.com/enerBydev/forgeproof.git
cd forgeproof
nix develop
python3 scripts/generate-fixtures.py
cargo run --locked -p engctl -- verify --bundle .local/fixtures/valid --trust .local/fixtures/trust.json --revocations .local/fixtures/current.json --target codex --json
```

Los fixtures son exclusivamente de prueba. Sus claves privadas se generan temporalmente y se eliminan; su autorización vence después de una hora. El generador rechaza sobrescribir un directorio existente.

| Salida | Significado |
| --- | --- |
| `0` | Integridad comprobada y autorización aprobada según las entradas locales |
| `1` | Paquete inválido, denegado o revocado |
| `2` | Integridad comprobada; vigencia desconocida, sin autorización |

El visor web interpreta el JSON en el navegador. **Un recibo importado no es evidencia autenticada ni un permiso reutilizable.** La CLI debe volver a verificar el paquete antes de utilizarlo.

## Verificar el desarrollo

Dentro de `nix develop`, después de generar los fixtures:

```sh
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
npm ci --ignore-scripts
npm test
npm run build
```

`nix flake check --no-update-lock-file --print-build-logs` construye el paquete y ejecuta la aceptación en una VM NixOS. Requiere Linux x86-64 con virtualización disponible. Las dependencias se fijan en `Cargo.lock` y `flake.lock`; el CI ordinario no las actualiza.

## Código y decisiones

- [`docs/usage/verify.md`](docs/usage/verify.md): contratos, uso y límites de confianza.
- [`docs/acceptance/verifier.md`](docs/acceptance/verifier.md): matriz de aceptación y alcance.
- [`docs/evidence/progress.md`](docs/evidence/progress.md): evidencia del desarrollo y revisión.
- [`docs/evidence/initial-toolchain.md`](docs/evidence/initial-toolchain.md): versiones y bloqueos.
- [`docs/design/architecture.md`](docs/design/architecture.md): diseño y entregas futuras.
- `crates/contracts`, `crates/catalog`, `apps/engctl`: dominio y CLI Rust.
- `apps/web`: inspector estático, sin dependencias JavaScript externas.

Las credenciales y claves privadas de producción pertenecen a la configuración externa, nunca a Git ni al almacén público de Nix.
