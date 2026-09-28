# Distribución local v1

Extiende la entrega 2 de `architecture.md`. Plataforma inicial: Linux x86-64; Rust 1.98.0 y locks existentes. No agrega servicios, credenciales ni dependencias externas nuevas.

## Contrato

- `engctl install --project DIR --bundle DIR --trust FILE --revocations FILE --target ID [--json]` vuelve a comprobar firma, inventario, hashes, confianza, destino y revocación actual antes de modificar el proyecto. Una aprobación previa no se reutiliza.
- Perfiles exactos iniciales: `codex@0.158.0` → `AGENTS.md`; `claude-code@2.1.283` → `CLAUDE.md`. Se rechazan versiones o roles sin adaptador. Los únicos roles activables en v1 son `instructions` y `policy`; todos los demás provocan incompatibilidad explícita.
- Renderizado determinista, orden por ruta, texto UTF-8 sin NUL, encabezado con publicación/hash/destino y contenido original entre separadores. Límite total: 32768 bytes por archivo, incluido el encabezado. No recorta instrucciones.
- El proyecto debe existir, sin enlaces simbólicos en su ruta. Un archivo nativo preexistente no administrado es un conflicto, incluso si sus bytes coinciden. Un archivo administrado modificado o ausente bloquea la instalación. `AGENTS.override.md` bloquea la instalación Codex para evitar un sombreado conocido.
- Cada proyecto puede tener ambos perfiles. El estado `.forgeproof/state.json` registra el perfil, publicación, hash del manifiesto y hash de cada salida. Instalar lo mismo dos veces devuelve `unchanged` sin reescribir salidas ni estado. Actualizar sustituye solamente el archivo del perfil seleccionado y conserva los demás.
- `.forgeproof` es un espacio reservado local. Solo acepta archivos de control conocidos; se rechazan enlaces, archivos especiales y enlaces duros en superficies administradas. Los nombres de destino son constantes, nunca rutas proporcionadas por el journal.

## Transacción y recuperación

Un bloqueo de archivo del sistema operativo serializa todos los comandos de distribución. El bloqueo se libera al terminar o morir el proceso. No se elimina el archivo del bloqueo.

Secuencia durable: validar estado y conflictos → escribir journal con imágenes anterior/posterior → reemplazar archivo nativo → reemplazar estado → eliminar journal. Cada reemplazo usa archivo temporal, `sync_all`, rename y sincronización del directorio. La actualización de dos archivos no se anuncia como atómica.

`engctl installation-status --project DIR [--json]` comprueba estado y hashes bajo el mismo bloqueo. Journal presente implica `installation_incomplete`; ningún consumidor administrado debe iniciar un trabajo en ese estado. Este comando demuestra consistencia local, no autorización vigente ni comportamiento del modelo.

`engctl recover --project DIR [--json]` revierte una transacción incompleta a sus imágenes anteriores. Antes de escribir, comprueba que ambos archivos todavía equivalgan a su imagen anterior o posterior. Cualquier tercera versión es un cambio local: rechaza la recuperación sin sobrescribirlo. Una reversión interrumpida es repetible. Si la primera instalación se revierte, elimina únicamente sus archivos creados; conserva el directorio de control y su bloqueo. Sin journal, `recover` es un no-op.

Entradas JSON de estado limitadas a 16 KiB; journal a 512 KiB; estructuras estrictas y validación de sus relaciones antes de recuperar. Salida JSON separada de los recibos del verificador: operación, estado y errores. Códigos: 0 para éxito; 1 para rechazo/error, incluida vigencia desconocida. El contrato de `verify` 0/1/2 permanece intacto.

## Prueba y límites

Se prueban procesos reales: primera instalación, repetición, actualización, ambos adaptadores, rechazo de paquetes no autorizados, versiones/roles incompatibles, archivos propios, edición local, symlinks, journal inválido y contención. Un arnés compila copias desechables con interrupciones deliberadas después de cada escritura durable; el binario original debe detectar el estado incompleto y recuperar sin perder cambios del usuario. CI y VM NixOS ejecutan los caminos publicados.

El operador controla el proyecto, staging, confianza, revocaciones y reloj. No hay escritores externos concurrentes durante el comando; el bloqueo coordina procesos Forgeproof, no editores ajenos. Fallos de procesos se prueban; corrupción arbitraria del disco y todos los posibles cortes eléctricos no se certifican. Los metadatos locales no son autoridad criptográfica. Los archivos generados no amplían permisos del cliente.

Estos perfiles son adaptadores de formato basados en documentación, no certificaciones de consumo efectivo. Configuración personal, instrucciones heredadas, contexto disponible y comportamiento de modelos requieren ensayos con clientes reales autenticados. No se inicia un cliente ni se instala software del cliente automáticamente. MCP, skills ejecutables y especialistas evaluados corresponden a entregas posteriores.

## Fuentes y decisiones — consultadas 2026-09-28

- OpenAI: https://learn.chatgpt.com/docs/agent-configuration/agents-md — descubrimiento de `AGENTS.md`, prioridad del override y presupuesto combinado predeterminado de 32 KiB. Nuestro límite local no garantiza el presupuesto restante de una sesión.
- Anthropic: https://code.claude.com/docs/en/memory — ubicación de `CLAUDE.md`, instrucciones jerárquicas y diferencias frente a configuración impuesta. Conservamos el adaptador explícito aunque versiones recientes también lean AGENTS.md.
- Versiones publicadas: https://github.com/openai/codex/releases/tag/rust-v0.158.0 y https://github.com/anthropics/claude-code/releases/tag/v2.1.283. La presencia de una versión publicada no constituye prueba de interoperabilidad.
- Rust: https://doc.rust-lang.org/std/fs/struct.File.html — bloqueo disponible desde 1.89, liberación por cierre y sincronización explícita. Se mantiene la toolchain fijada en el primer hito.
