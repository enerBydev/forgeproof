use crate::{VerificationError, VerifiedBundle};
use eng_contracts::{
    AuthorizationStatus, IntegrityStatus, VerificationReceipt, valid_hash, valid_id,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct RevocationSnapshot {
    document: RevocationDocument,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RevocationDocument {
    schema_version: u32,
    generated_at: u64,
    expires_at: u64,
    revoked_release_ids: Vec<String>,
    revoked_manifest_sha256: Vec<String>,
    revoked_signer_ids: Vec<String>,
}
impl RevocationSnapshot {
    pub fn parse(bytes: &[u8]) -> Result<Self, VerificationError> {
        let invalid = || VerificationError::new("invalid_revocations", None);
        if bytes.len() > 1024 * 1024 {
            return Err(invalid());
        }
        let document: RevocationDocument = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        if document.schema_version != 1 || document.generated_at >= document.expires_at {
            return Err(invalid());
        }
        let lists = [
            &document.revoked_release_ids,
            &document.revoked_manifest_sha256,
            &document.revoked_signer_ids,
        ];
        if lists.iter().map(|v| v.len()).sum::<usize>() > 10000 {
            return Err(invalid());
        }
        for (i, list) in lists.iter().enumerate() {
            let unique: BTreeSet<_> = list.iter().collect();
            if unique.len() != list.len()
                || !list
                    .iter()
                    .all(|s| if i == 1 { valid_hash(s) } else { valid_id(s) })
            {
                return Err(invalid());
            }
        }
        Ok(Self { document })
    }
}
pub fn authorize(
    bundle: &VerifiedBundle,
    target: &str,
    revocations: Option<&RevocationSnapshot>,
    now: SystemTime,
) -> VerificationReceipt {
    let mut receipt = VerificationReceipt {
        schema_version: 1,
        release_id: Some(bundle.manifest().release_id.clone()),
        manifest_sha256: Some(bundle.manifest_sha256().into()),
        signer_id: Some(bundle.signer_id().into()),
        target: target.into(),
        integrity: IntegrityStatus::Verified,
        authorization: AuthorizationStatus::Unknown,
        errors: vec![],
    };
    let decision = if !bundle
        .manifest()
        .supported_targets
        .iter()
        .any(|t| t == target)
    {
        (AuthorizationStatus::Denied, Some("unsupported_target"))
    } else if let Some(snapshot) = revocations {
        let d = &snapshot.document;
        if d.revoked_release_ids
            .contains(&bundle.manifest().release_id)
            || d.revoked_manifest_sha256
                .iter()
                .any(|v| v == bundle.manifest_sha256())
            || d.revoked_signer_ids.iter().any(|v| v == bundle.signer_id())
        {
            (AuthorizationStatus::Revoked, Some("release_revoked"))
        } else if let Ok(time) = now.duration_since(UNIX_EPOCH) {
            if time.as_secs() < d.generated_at {
                (
                    AuthorizationStatus::Unknown,
                    Some("revocations_not_yet_valid"),
                )
            } else if time.as_secs() >= d.expires_at {
                (AuthorizationStatus::Unknown, Some("revocations_expired"))
            } else {
                (AuthorizationStatus::Approved, None)
            }
        } else {
            (AuthorizationStatus::Unknown, Some("clock_invalid"))
        }
    } else {
        (AuthorizationStatus::Unknown, Some("revocations_missing"))
    };
    receipt.authorization = decision.0;
    if let Some(code) = deci…16016 tokens truncated…a y exige error de hash en la siguiente.
- [x] Ejecutar los tests de `eng-catalog` y comprobar sus fallos iniciales.
- [x] Implementar autorización separada de integridad. Un target no listado genera `unsupported_target` y `AuthorizationStatus::Denied`; se devuelve un recibo no utilizable, sin degradación silenciosa.
- [x] Ejecutar hasta PASS, incluyendo el instante exacto de expiración y entradas de revocación duplicadas rechazadas durante su lectura.
- [x] Registrar el cambio en Git después de revisar que la firma válida por sí sola no produce autorización.

### Task 4: CLI y ensayo de aceptación NixOS

**Files:** crear `apps/engctl/Cargo.toml` (paquete `engctl`), `apps/engctl/src/main.rs`, `apps/engctl/tests/verify_cli.rs`, `nix/tests/verifier.nix`, `docs/usage/verify.md` y `docs/acceptance/verifier.md`; actualizar workspace, flake y bloqueos.

**Interfaces:** consumir parser, `verify_integrity` y `authorize`; producir la CLI, su JSON versionado y códigos de salida 0, 1 y 2 definidos en el contrato. Para errores anteriores a leer un release válido, `release_id`, hash y firmante pueden ser `null`.

- [x] Escribir aceptación del comando con paquete aprobado, alterado, revocado, firmante no confiable, target incompatible y registro de revocación vencido. Comprobar JSON, código de salida y ausencia de cambios en los archivos de entrada.
- [x] Ejecutar `cargo test -p engctl --locked` y registrar los casos que fallan antes de conectar los componentes.
- [x] Implementar CLI y serialización; stdout contiene únicamente JSON cuando se usa `--json`, mientras los diagnósticos auxiliares van a stderr.
- [x] Ejecutar `cargo test --workspace --locked`, `cargo fmt --all -- --check` y los lints seleccionados para el workspace.
- [x] Añadir un test NixOS con el paquete y sus dependencias declaradas que ejecute la misma aceptación desde una VM limpia. Exponerlo como `checks.x86_64-linux.verifier`.
- [x] Ejecutar `nix flake check` en un runner con virtualización disponible. Conservar configuración, revisión, logs y resultados; si esa capacidad no existe, el hito queda pendiente de esa comprobación.
- [x] Revisar el diff y la matriz de aceptación; registrar el cambio en Git y producir el informe del hito. No etiquetar el resto de la plataforma como implementado por haber completado este verificador.

El segundo hito se planifica después de revisar esta evidencia: renderizado por cliente, instalación, detección de modificaciones y recuperación. La actualización de varios archivos no se considera atómica por usar escrituras atómicas individuales; se define y prueba un journal de recuperación y se bloquean nuevos trabajos administrados mientras la instalación esté incompleta.

## Fuentes oficiales consultadas

Estas fuentes sustentan capacidades documentadas. La arquitectura, prioridades, umbrales y división de trabajo son propuestas de este documento. Sus enlaces se consultaron el 26 de septiembre de 2026; al implementar se conservan snapshots de las versiones elegidas.

- **S1.** [SDK oficial Rust de MCP](https://github.com/modelcontextprotocol/rust-sdk).
- **S2.** [PostgreSQL: búsqueda de texto completo](https://www.postgresql.org/docs/current/textsearch.html).
- **S3.** [PostgreSQL: SELECT y SKIP LOCKED](https://www.postgresql.org/docs/current/sql-select.html).
- **S4.** [SQLx](https://github.com/transact-rs/sqlx).
- **S5.** [Minisign](https://jedisct1.github.io/minisign/).
- **S6.** [Pruebas de integración con máquinas virtuales NixOS](https://nix.dev/tutorials/nixos/integration-testing-using-virtual-machines.html).
- **S7.** [Nix: flakes y archivos de bloqueo](https://nix.dev/manual/nix/stable/command-ref/new-cli/nix3-flake).
- **S8.** [pgvector: búsqueda exacta, índices y filtrado](https://github.com/pgvector/pgvector).
- **S9.** [SQLite: usos apropiados y concurrencia](https://www.sqlite.org/whentouse.html).
- **S10.** [Arquitectura de Supabase](https://supabase.com/docs/guides/getting-started/architecture).
- **S11.** [Consultas híbridas de Qdrant](https://qdrant.tech/documentation/search/hybrid-queries/).
- **S12.** [Conformidad de MCP](https://github.com/modelcontextprotocol/conformance).
- **S13.** [Autorización de MCP](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization).
- **S14.** [Cargo: cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html).
- **S15.** [Proptest](https://proptest-rs.github.io/proptest/intro.html).
- **S16.** [Rust Fuzz Book: cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html).
- **S17.** [OpenAI: evaluación sistemática de skills](https://developers.openai.com/blog/eval-skills).
- **S18.** [Tokio](https://tokio.rs/).
- **S19.** [MCP en Codex y ChatGPT](https://learn.chatgpt.com/docs/extend/mcp).
- **S20.** [MCP en Claude Code](https://code.claude.com/docs/en/mcp).
- **S21.** [Conectores MCP remotos en Claude](https://support.claude.com/en/articles/11175166-get-started-with-custom-connectors-using-remote-mcp).

## Revisión del documento

Se comprobaron las correspondencias entre objetivos y entregas, la separación entre integridad y autorización, el uso de nombres de interfaces en el primer hito, las fuentes de las capacidades afirmadas y la distinción entre metas y resultados. Esta sección describe la revisión original del diseño. El estado posterior del primer hito y sus comprobaciones está registrado en la trazabilidad; no se atribuyen esos resultados al resto de la plataforma.
