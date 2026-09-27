# Aceptación del primer hito

Alcance: contratos v1, autenticidad e integridad de paquetes, autorización local, CLI, entorno NixOS y visor de recibos. Las entradas de confianza pertenecen al operador y el staging permanece inmutable durante la lectura.

| Condición | Evidencia implementada |
| --- | --- |
| Límites y estructura del manifiesto | 7 tests de contratos, incluidos límites exactos de cantidad y tamaño |
| Firma válida de una clave no confiable | Rechazo `untrusted_signer`, fixture firmado por otra clave real |
| Manipulación de manifiesto o contenido | Rechazo de firma o SHA-256, incluso después de una verificación anterior |
| Rutas de escape, enlaces y contenido adicional | Validación de rutas y tests de symlink, archivos, directorios y ausencia de archivos |
| Revocación por publicación, hash o firmante | Test parametrizado; salida 1 |
| Registro ausente, futuro o vencido | Vigencia desconocida; salida 2, incluido el instante exacto de vencimiento |
| Cliente no declarado | Denegación `unsupported_target` |
| Interfaz de comando | 6 tests con procesos reales, JSON, códigos de salida y manifiesto sin cambios |
| NixOS limpio | VM declarada en `nix/tests/verifier.nix`, 7 escenarios de CLI |
| Recibo importado | 9 tests web; nunca se convierte en evidencia autenticada |
| JSON con coerción de tipos | Regresión del hallazgo de revisión: arrays y objetos no pueden representar autorización |

Los resultados observados y revisiones correspondientes están en [el registro de evidencia](../evidence/progress.md) y en [GitHub Actions](https://github.com/enerBydev/forgeproof/actions). Una fila describe cobertura implementada; únicamente una ejecución satisfactoria de esa revisión demuestra que pasó.

## Revisión independiente

La revisión de código encontró un defecto importante en el parser web: JavaScript convertía un array en nombre de propiedad y podía eludir una comprobación de estados contradictorios. Se reprodujo con una prueba fallida; una comprobación explícita de tipo corrigió el fallo. También se cubre el objeto que provocaba una excepción durante esa conversión. No se encontraron otros bloqueantes de Rust dentro del modelo de confianza declarado.

El revisor no ejecutó Cargo ni Nix en su entorno. La evidencia de compilación, lint y sistema procede del CI, no de esa revisión documental.

## Límites de esta entrega

No certifica la calidad de las instrucciones, el comportamiento de un modelo, la interoperabilidad MCP, la instalación en los clientes ni la actualización autónoma. No protege contra un operador hostil que sustituya la confianza o modifique concurrentemente el staging. Un recibo no está firmado y nunca se reutiliza como permiso de ejecución.

El siguiente hito es el renderizado e instalación por cliente, con detección de cambios y recuperación de instalaciones incompletas. El catálogo remoto, la documentación versionada y las evaluaciones de recuperación se incorporan después sobre estos contratos.
