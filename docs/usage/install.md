# Instalar instrucciones verificadas

El instalador local admite Linux x86-64 y estos perfiles de formato exactos:

| Destino | Salida en el proyecto |
| --- | --- |
| `codex@0.158.0` | `AGENTS.md` |
| `claude-code@2.1.283` | `CLAUDE.md` |

El paquete firmado debe declarar ese destino exacto. Solo se activan roles `instructions` y `policy`; otros roles y versiones se rechazan. Esto verifica distribución de archivos. Todavía no certifica consumo efectivo en clientes autenticados ni equivalencia del comportamiento del modelo.

Desde el entorno Nix del repositorio:

```sh
mkdir -p /ruta/al/proyecto
engctl install --project /ruta/al/proyecto \
  --bundle /ruta/al/paquete --trust /ruta/a/confianza.json \
  --revocations /ruta/a/revocaciones.json --target codex@0.158.0 --json
engctl installation-status --project /ruta/al/proyecto --json
```

Repetir el comando con el mismo paquete devuelve `unchanged` y conserva los archivos existentes. Cambiar `--bundle` actualiza ese cliente después de volver a verificar firma, contenido y autorización vigente. Se pueden instalar ambos perfiles en el mismo proyecto. Instala desde una copia estable del paquete, con confianza y revocaciones controladas por ti.

Los archivos nativos deben estar ausentes en la primera instalación. Si ya mantienes un `AGENTS.md` o `CLAUDE.md`, el instalador rechaza apropiárselo: integra deliberadamente tus instrucciones en el paquete antes de retirar tu archivo original. No existe una opción `--force`. Las ediciones locales, archivos eliminados, enlaces, un override de Codex y un estado inválido bloquean la operación.

El contenido generado tiene un límite de 32768 bytes e incluye identidad de publicación y procedencia. El presupuesto real del cliente también depende de instrucciones heredadas y configuración personal; el límite del archivo por sí solo no prueba que todo llegue al modelo. Reinicia la sesión del cliente tras cambiar instrucciones.

## Recuperar una interrupción

```sh
engctl recover --project /ruta/al/proyecto --json
engctl installation-status --project /ruta/al/proyecto --json
```

Un journal pendiente hace que `installation-status` e instalaciones nuevas fallen con `installation_incomplete`. Recuperar revierte al estado anterior; después puedes reintentar instalar. Si editaste un archivo durante la interrupción, la recuperación devuelve `local_modification` y conserva los bytes. Guarda tus cambios aparte y reconcilia el archivo con su imagen anterior o posterior antes de reintentar; nunca elimines el journal para aparentar una instalación completa.

`.forgeproof` contiene estado, journal temporal y bloqueo persistente. No borres el bloqueo ni copies el directorio de control entre proyectos. El journal desaparece al terminar. El estado no es un recibo firmado ni una autorización reutilizable: el futuro ejecutor deberá comprobar consistencia y volver a autorizar el paquete antes de cada trabajo.

Los comandos de distribución emiten JSON con `operation`, `status` y `errors`, y devuelven 0 en éxito o 1 en rechazo/error. `installation-status` solo confirma integridad de los archivos administrados; no consulta revocaciones. El formato y las salidas 0/1/2 de `engctl verify` no cambian.
