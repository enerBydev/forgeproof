# Herramientas fijadas

Selección inicial: 26–27 de septiembre de 2026. La fuente de verdad ejecutable son los archivos de bloqueo y la revisión Git, no la palabra «latest».

| Componente | Selección |
| --- | --- |
| Rust | 1.98.0, perfil minimal, rustfmt y Clippy |
| Plataforma inicial | x86_64-linux |
| nixpkgs | `5e2305d577ca00acbba631b05cb1094d172b29f3`, procedente de NixOS 26.05 |
| rust-overlay | `edea2d6f98b1e5f64ce5ac21e19a96c603c20b8c` |
| serde | 1.0.229 |
| serde_json | 1.0.151 |
| sha2 | 0.11.0 |
| minisign-verify | 0.2.5 |
| Node para la web | Familia 24; CI fija 24.19.0, Nix fija el derivado de nixpkgs |

`Cargo.lock` fue producido por Cargo en CI y revisado antes de incorporarlo. `flake.lock` fue producido por Nix; sus hashes de contenido no se inventaron. Rust se obtiene del manifiesto oficial a través del overlay fijado. Nix fija también Python, Minisign y las bibliotecas del sistema utilizadas por la construcción y la VM.

Vercel gestiona el parche de su runtime Node 24. El build de la interfaz no tiene dependencias externas ni compila el núcleo Rust. No se atribuye reproducibilidad de todo el proveedor cloud al archivo de bloqueo de Nix.

La generación de fixtures usa el ejecutable independiente Minisign. Sus claves aleatorias y firmas cambian en cada ejecución; contenido, significado y resultados esperados son equivalentes. Esto no es una afirmación de builds idénticos bit a bit: esa propiedad requiere una comparación específica posterior.
