# Trazabilidad del plan de ingeniería

Referencia: `docs/design/architecture.md`, propuesta original de 2026-09-26. Actualización: 2026-09-28 UTC. Una prueba aprobada demuestra exclusivamente la condición que ejercita.

## Estado por entrega

| Entrega | Estado | Alcance demostrado |
| --- | --- | --- |
| 1. Publicaciones verificables | Implementada y verificada | Contratos, integridad, firma, revocación, CLI y VM NixOS |
| 2. Distribución local | Siguiente entrega | Instalador y adaptadores con idempotencia, conflictos y recuperación |
| 3. Contrato y entorno de proyectos | Pendiente | El flake del propio verificador no valida todavía proyectos consumidores |
| 4. Servicio y documentación | Pendiente | PostgreSQL, MCP, snapshots y recuperación textual |
| 5. Ejecución y clientes remotos | Pendiente | Trabajos, OAuth y pruebas reales en Claude/ChatGPT |
| 6. Recuperación evaluada | Pendiente | Benchmark y decisión sobre vectores |
| 7. Actualizaciones controladas | Pendiente | Evaluación, promoción y reversión |
| 8. Ampliación del catálogo | Pendiente | Especialistas con evaluaciones propias |

## Primer hito: requisito → implementación → prueba → evidencia

| Requisito | Implementación y pruebas | Resultado observado |
| --- | --- | --- |
| Esquema, rutas, duplicados y límites | `crates/contracts/src/manifest.rs`; suite `manifest` | 8 pruebas; acepta exactamente 1 MiB y rechaza un byte adicional |
| Firma confiable sobre bytes originales | `crates/catalog/src/verify.rs`; suite `integrity`; `scripts/generate-fixtures.py` | 12 pruebas; oráculo Minisign externo acepta dos firmas con sus claves y rechaza clave incorrecta, firma corrupta y manifiesto modificado |
| Contenido exacto | Suite `integrity` | Hash, tamaño, ausencia, extras, symlink y cambio posterior detectados |
| Revocación y vigencia independientes de integridad | `crates/catalog/src/authorize.rs`; suite `authorization` | 6 pruebas; revocación por ID/hash/firmante, expiración exacta, futuro y ausencia |
| CLI, JSON y salidas 0/1/2 | `apps/engctl/tests/verify_cli.rs` | 6 pruebas con procesos reales |
| Ninguna entrada modificada | Instantánea recursiva antes/después en `verify_cli.rs` | Compara nombres, tipos, bytes y destinos de enlaces de todo el árbol de fixtures: paquete, firma, confianza y revocaciones |
| Sensibilidad de las nuevas pruebas | `scripts/check-verifier-mutations.py` | Detecta rechazo incorrecto de 1 MiB y escritura posterior sobre manifiesto, firma, contenido, confianza y revocaciones; los originales restaurados pasan |
| Entorno fijado y VM limpia | `flake.lock`, `Cargo.lock`, `rust-toolchain.toml`, `nix/tests/verifier.nix` | Construcción congelada y 7 escenarios de aceptación en VM NixOS |
| Revisión y evidencia | `docs/evidence/progress.md`, PRs y artefactos CI | Fallos iniciales, correcciones, revisión y resultados conservados |

Evidencia de cierre de brechas: [CI 36374905582](https://github.com/enerBydev/forgeproof/actions/runs/36374905582), revisión `2bbf8d077a6db5e1209b4f23671c947879702353`. 32 pruebas Rust, 9 del visor, 6 mutaciones detectadas y comprobación externa de las firmas. La VM ejecutó sus 7 escenarios. Los registros de CI identifican la revisión exacta.

## Adaptaciones explícitas frente a las rutas propuestas

- Los casos de `authenticity.rs` y `revalidation.rs` están en `crates/catalog/tests/integrity.rs`. Se preservan los comportamientos exigidos; no se requieren archivos separados para probarlos.
- `tests/fixtures/bundles` y `tests/fixtures/trust` se sustituyen por un generador versionado que crea `.local/fixtures`. Las claves de prueba son temporales. Los bytes del contenido y sus resultados son equivalentes, pero las claves y firmas aleatorias no son idénticas entre ejecuciones.
- El inspector estático de Vercel es una ampliación solicitada después del diseño inicial. Solo interpreta recibos locales; no sustituye a la CLI ni implementa MCP.
- La publicación inicial usó squash. El contenido integrado coincide con el de la rama original; los identificadores de los commits de desarrollo son diferentes. La limpieza de ramas se realiza después de verificar el contenido integrado.

## Límites que permanecen explícitos

El primer hito supone staging inmutable y confianza, revocaciones y reloj controlados por el operador. La declaración de un cliente en el manifiesto no certifica sus capacidades nativas. La batería actual no incluye todavía fuzzing ni pruebas por propiedades de toda la plataforma, snapshots documentales completos ni evaluación de especialistas; esas actividades siguen pendientes dentro del programa de verificación global.
