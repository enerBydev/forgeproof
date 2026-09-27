# Verificar un paquete

`engctl` inspecciona archivos locales. No descarga, instala ni ejecuta el contenido del paquete.

```sh
nix build --no-update-lock-file
./result/bin/engctl verify \
  --bundle /ruta/staging/paquete \
  --trust /ruta/operador/trust.json \
  --revocations /ruta/operador/revocations.json \
  --target codex \
  --json
```

El comando emite un recibo JSON en stdout, también cuando la validación falla. `--json` es explícito pero opcional. `--help` describe el comando. Los argumentos desconocidos, duplicados o incompletos producen `invalid_arguments` y salida 1.

## Entradas y confianza

El operador proporciona `trust.json` y el registro de revocaciones por un canal confiable, fuera del paquete. Un agente o productor de paquetes no debe poder reemplazar esas entradas ni el reloj del verificador. La firma de registros remotos y la protección contra retrocesos de versión se incorporarán con el catálogo remoto.

El staging debe permanecer inmutable durante la verificación. Se rechazan enlaces simbólicos, rutas con `..`, archivos especiales y contenido adicional. Esto no impide que un administrador hostil cambie el sistema concurrentemente. El recibo describe los bytes inspeccionados: si cambian o pasa tiempo, hay que volver a verificar antes de ejecutar una tarea.

Registro de confianza v1:

```json
{
  "schema_version": 1,
  "keys": [{"id": "publisher", "public_key": "BASE64_DE_LA_CLAVE_PUBLICA_MINISIGN"}]
}
```

`public_key` contiene la línea base64 de una clave pública Minisign, sin la línea de comentario. El texto anterior es un marcador de formato, no una clave utilizable. Las claves e identificadores duplicados se rechazan. Límite: 64 KiB y 128 claves.

Registro de revocaciones v1:

```json
{
  "schema_version": 1,
  "generated_at": 100,
  "expires_at": 200,
  "revoked_release_ids": [],
  "revoked_manifest_sha256": [],
  "revoked_signer_ids": []
}
```

Las fechas son segundos Unix UTC. El ejemplo está deliberadamente vencido; el operador establece la vigencia real. Se exige `generated_at < expires_at`, listas sin duplicados y un máximo total de 10 000 entradas y 1 MiB. La revocación conocida siempre impide aprobar, incluso si el registro ha vencido. Un registro ausente, vencido o futuro produce vigencia desconocida. Un registro presente pero mal formado se rechaza.

## Paquete y firma

Un paquete contiene `manifest.json`, su firma `manifest.json.minisig` y exclusivamente los archivos enumerados. El manifiesto declara `schema_version`, `release_id`, `supported_targets` y `files`; cada archivo tiene `path`, `sha256`, `size_bytes` y `role`. Roles: `instructions`, `skill`, `policy`, `source_spec`, `test_definition`.

La firma verifica los bytes originales del manifiesto. Cambiar espacios o saltos de línea después de firmarlo invalida la firma. No se aceptan firmas Minisign heredadas sin prehash. Cada hash SHA-256 comprueba el contenido original del archivo.

Límites: manifiesto de 1 MiB; entre 1 y 1024 archivos; 16 MiB por archivo; 256 MiB en total; rutas relativas UTF-8 de hasta 4096 bytes, 32 componentes y 255 bytes por componente. Los identificadores tienen de 1 a 128 caracteres ASCII alfanuméricos o `._@-`; los hashes son hexadecimales minúsculos de 64 caracteres. Los esquemas v1 rechazan campos desconocidos.

`supported_targets` es una declaración firmada del productor que el verificador compara exactamente. No demuestra que el paquete funcione en ese cliente; la compatibilidad funcional exige pruebas independientes de su adaptador.

## Interpretar el recibo

| Integridad | Autorización | Salida | Acción del consumidor |
| --- | --- | --- | --- |
| `verified` | `approved` | 0 | Puede continuar bajo la política local y el supuesto de staging inmutable |
| `verified` | `unknown` | 2 | Obtener revocaciones vigentes; no ejecutar |
| `verified` | `revoked` o `denied` | 1 | No ejecutar |
| `failed` | `denied` | 1 | Corregir la entrada o rechazar el paquete |

Los fallos previos a comprobar la identidad pueden dejar `release_id`, `manifest_sha256` y `signer_id` en `null`. `errors` contiene códigos estables y, cuando corresponde, la ruta afectada. La identidad comprobada no se pierde si luego falla la lectura del registro de revocaciones.

El visor de Vercel acepta recibos de hasta 64 KiB y valida su estructura. Todo el procesamiento ocurre localmente en el navegador. El visor no posee el paquete original ni autentica el recibo; las etiquetas muestran afirmaciones importadas. No debe utilizarse como control de autorización.
