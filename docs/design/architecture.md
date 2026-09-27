# Plataforma de ingeniería verificable — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement the first milestone task-by-task, or superpowers:subagent-driven-development if parallel agent execution is explicitly selected. Steps use checkbox (`- [ ]`) syntax for tracking. Do not treat this document as evidence that the implementation or its tests already exist.

**Goal:** Construir una plataforma personal, ampliable a un equipo, que distribuya especialistas exactos, aporte documentación aplicable al proyecto y verifique el software producido desde Codex, Claude Code, Claude y ChatGPT.

**Architecture:** Servicio modular en Rust, PostgreSQL como almacén operativo y catálogo de publicaciones inmutables procedentes de Git. Los clientes consumen un MCP común; adaptadores específicos distribuyen sus instrucciones. Un ejecutor NixOS aplica contratos de entorno y recoge evidencias. La recuperación semántica se incorpora cuando supera una evaluación contra la búsqueda inicial.

**Tech Stack:** Rust, Tokio, rmcp, SQLx, PostgreSQL, Nix/NixOS, Git, Minisign y almacenamiento de objetos por hash. pgvector es una ampliación condicionada a evaluación; Supabase es una alternativa de alojamiento y servicios; ninguno es requisito del dominio.

**Spec:** La especificación está incluida en este mismo documento, desde «Especificación y decisiones» hasta «Plan de entregas». El plan ejecutable del primer hito figura al final. Las rutas de código de ese plan son archivos futuros, no archivos ya creados.

**Estado:** Propuesta técnica de 26 de septiembre de 2026. Se contrastaron las capacidades documentadas que sustentan las decisiones. No se ha implementado, instalado ni certificado la plataforma. Las cifras de aceptación son objetivos propuestos, no resultados medidos.

## Global Constraints

- NixOS es la base del equipo y de la ejecución controlada; se empieza con `x86_64-linux`. Otras arquitecturas requieren su propia matriz de pruebas.
- Cada ejecución fija publicación, contrato de proyecto y snapshot documental antes de comenzar.
- Las instrucciones obligatorias se recuperan completas por identificador y hash.
- Una búsqueda semántica nunca determina por sí sola permisos ni versiones autorizadas.
- Un fallo de una comprobación obligatoria impide marcar la ejecución como conforme.
- Los originales de políticas, especialistas y pruebas se conservan en Git; las publicaciones aprobadas son inmutables.
- El proceso generador carece de la clave de publicación y de permisos para sustituir unilateralmente el conjunto protegido de aceptación.
- Los perfiles nativos no pueden elevar permisos sobre las restricciones del cliente anfitrión.
- La primera entrega es un verificador local y no ejecuta scripts de los paquetes que inspecciona.
- Las versiones concretas del stack se resuelven, prueban y fijan durante la primera tarea; este documento no inventa números de versión no comprobados.

## Review Focus

1. Una publicación firmada por una clave diferente de la autorizada debe rechazarse, aunque su firma sea matemáticamente válida.
2. Rutas con escape, enlaces simbólicos o archivos adicionales deben impedir que un paquete se considere exacto.
3. Una publicación revocada debe rechazarse; sin información de vigencia se distingue verificación de integridad de autorización para usarla.
4. Un cliente que no puede representar una capacidad obligatoria debe producir un error explícito de compatibilidad.
5. Una modificación de un archivo después de emitir el recibo debe detectarse antes de reutilizarlo para ejecutar trabajo.

Las tareas 2, 3 y 4 del primer hito asignan pruebas a estas cinco condiciones. El resto de riesgos aparece en la matriz de verificación de la plataforma completa.

## Especificación y decisiones

### 1. Resultado que se busca

Un trabajo conforme debe poder responder: qué se solicitó, qué publicación de especialistas se utilizó, qué reglas eran obligatorias, qué versiones reales se ejecutaron, qué documentación sustentó las decisiones y qué pruebas verificaron el resultado.

La reproducibilidad abarca entradas, artefactos, entorno y comprobaciones. Dos modelos o dos ejecuciones pueden producir cambios distintos que satisfagan el mismo contrato. No se promete identidad del razonamiento ni corrección universal por haber superado una batería finita.

La interfaz del usuario puede cambiar entre Codex, Claude Code, ChatGPT y Claude. La información autoritativa reside en el catálogo y el contrato; las verificaciones exigibles residen en el ejecutor y CI. Un agente nativo que trabaja fuera de ese flujo no recibe automáticamente la misma garantía.

### 2. Decisión de stack y alternativas

| Componente | Elección inicial | Razón y condición de cambio |
|---|---|---|
| Núcleo y CLI | Rust | Tipos explícitos para contratos, estados y errores; binarios distribuibles en Nix. No se interpreta esto como ausencia de errores de lógica. |
| Servicio MCP | rmcp sobre Tokio; HTTP para servicio compartido y stdio para desarrollo local | SDK oficial y frontera pequeña. Mantener protocolos y cliente fuera del dominio. [S1] |
| Base operativa | PostgreSQL | Relaciones, transacciones, permisos, múltiples consumidores y búsqueda textual en un mismo sistema. [S2, S3] |
| Acceso SQL | SQLx | Consultas SQL visibles y migraciones versionadas; comprobación de consultas donde resulte aplicable. [S4] |
| Catálogo original | Git más publicaciones firmadas | Historial revisable y distribución exacta. La base indexa publicaciones; no puede reescribirlas. |
| Archivos voluminosos | Objetos por SHA-256; filesystem en desarrollo y backend compatible con S3 para despliegue | Snapshots y trazas se verifican por contenido. Los índices se reconstruyen desde estos objetos. |
| Firmas | Minisign | Utilizar una implementación existente de firma y verificación de archivos. La clave pública confiable se instala fuera del paquete recibido. [S5] |
| Trabajos asíncronos | Worker separado y tabla de trabajos en PostgreSQL | Reintentos, leases e idempotencia explícitos. `SKIP LOCKED` puede servir para reclamar trabajos; no convierte la ejecución en exactamente una vez. [S3] |
| Entorno | Nix/NixOS y servicios systemd | Declarar herramientas, servicios y pruebas de sistema en VM. [S6, S7] |
| Búsqueda inicial | Versiones y símbolos exactos más búsqueda textual | Establece una referencia medible antes de introducir embeddings. [S2] |
| Búsqueda semántica | pgvector, si la evaluación demuestra mejora | Evita operar otra base para el primer volumen; conserva originales y metadatos. [S8] |
| Interfaz inicial | CLI y MCP | Permite comprobar el flujo principal. Una interfaz Nuxt/Vue se incorpora al necesitar navegación, revisión o administración visual. |

**SQLite:** buena opción para un producto exclusivamente local o una futura caché del instalador. Para el servicio compartido con ingestión, trabajos y varios clientes elegiría PostgreSQL. SQLite admite un escritor por archivo en cada instante; esto no implica que sea incapaz de servir aplicaciones multiusuario. [S9]

**Supabase:** opción válida si Auth, Storage, administración y alojamiento reducen trabajo operativo. Su núcleo es PostgreSQL. El dominio accederá mediante interfaces propias y SQL, para que esa elección no obligue a reescribir políticas o especialistas. Autoalojar todo Supabase añade servicios que la primera entrega no necesita. [S10]

**Qdrant:** alternativa para una necesidad medida de recuperación vectorial independiente. Compararía recall, filtrado, latencia, consumo y coste de operación con el mismo corpus y carga. La capacidad de consulta híbrida existe; su superioridad para este producto debe medirse. [S11]

La primera instalación compartida puede vivir en una VM NixOS con API, worker y PostgreSQL. Sus copias se conservan fuera de esa máquina y se prueban mediante restauración. Esto reduce operación inicial, pero no constituye alta disponibilidad. Un objetivo posterior de disponibilidad puede justificar separar o gestionar la base de datos.

### 3. Límites de módulos y confianza

Los módulos lógicos serán contratos, catálogo, políticas, conocimiento, adaptadores, ejecución y evaluación. Empiezan en un monorepositorio y un servicio modular, con un proceso worker. Los límites se convierten en servicios independientes únicamente cuando exista una necesidad operativa comprobada.

El coordinador elige especialistas según la tarea. Cada especialista declara qué sabe hacer, qué necesita, qué fuentes le corresponden, qué herramientas puede solicitar y qué entrega. Se cargan sólo los procedimientos relevantes. No hay un prompt permanente con toda la documentación de todos los ecosistemas.

Las filosofías y patrones se representan mediante fichas de decisión: problema, condiciones de aplicación, beneficios, costes, contraejemplos, evidencia y regla de prevalencia. Las políticas distinguen obligación, preferencia y recomendación condicionada. Por ejemplo, la adopción de CQRS requeriría una justificación ligada al proyecto, mientras que respetar su archivo de bloqueo sería obligatorio.

La documentación recuperada es evidencia externa. No adquiere autoridad para cambiar permisos, claves, políticas o criterios de aceptación. El proceso de actualización propone revisiones; la promoción utiliza controles distintos.

### 4. Modelo de información

| Registro | Campos esenciales |
|---|---|
| Publicación | ID, revisión Git, manifiesto, hashes, firma, clave firmante, estado de promoción/revocación |
| Especialista | ID, publicación, instrucciones completas, capacidades, skills, requisitos de herramientas |
| Contrato | Proyecto y commit, publicación fijada, políticas, hashes de lockfiles, snapshots documentales |
| Fuente | URL/repositorio oficial, producto, revisión/tag, licencia, fecha de captura, hash y estado de vigencia |
| Fragmento | Fuente y sección, símbolo/API, versiones aplicables, texto, idioma; embedding opcional con modelo y dimensiones |
| Trabajo | Contrato, petición, clave de idempotencia, estado, lease, intentos, límites de tiempo y recursos |
| Ejecución | Cliente y modelo observado, herramientas efectivas, fuentes usadas, eventos y artefactos |
| Evaluación | Casos y versión del evaluador, resultado, métricas, fallos y evidencias |

Las claves foráneas, restricciones de unicidad y transacciones comprueban lo estructural. La autorización se aplica en el servicio y en el acceso a datos. Las cachés de recuperación incluyen identidad/proyecto, snapshot y versión del índice para evitar reutilizar resultados de otro contexto.

La memoria de conversaciones o experiencias del equipo se mantiene separada de políticas y documentación oficial. Una observación puede convertirse en propuesta revisable; no altera por sí sola una regla aprobada.

### 5. Documentación exhaustiva con un alcance comprobable

La aspiración de consultar todas las fuentes pertinentes se convierte en un inventario finito. Se enumeran el stack directo, las dependencias transitivas y las interfaces que realmente se usan. Leer todos los textos asociados a cada dependencia y doctrina no es un criterio verificable de comprensión ni un requisito que garantice corrección.

Para cada cambio se exige una matriz `requisito → versión → fuente/sección → decisión → prueba → resultado`. La cobertura se declara respecto a ese inventario. Se registran fuentes ausentes o no versionadas; no se declara lectura o verificación de algo que no se consultó.

El proceso documental será:

1. Resolver versiones desde las restricciones del proyecto y registros oficiales; comprobar el grafo de compatibilidad.
2. Inventariar dependencias directas y transitivas mediante los gestores de paquetes, conservando una lista de componentes y versiones.
3. Capturar documentación, guías de migración y especificaciones relevantes desde revisiones identificables. Respetar licencias y límites de acceso.
4. Conservar originales y extraer texto, encabezados, símbolos y ejemplos sin ejecutar scripts encontrados en páginas.
5. Para cada funcionalidad, recuperar las secciones necesarias y contrastarlas con tipos, esquemas, exports o código de la versión instalada.
6. Convertir los ejemplos importantes en pruebas reproducibles y registrar discrepancias entre manual y comportamiento.

Puede capturarse un manual completo cuando aporte valor. La evidencia final seguirá nombrando las secciones efectivamente utilizadas. Los textos de arquitectura se evalúan por aplicabilidad, no por su fecha de publicación únicamente.

### 6. Recuperación y decisión sobre vectores

Se selecciona primero el conjunto autorizado por proyecto, tecnología, versión y snapshot. Se busca por identificadores exactos y texto; una fase semántica, si se habilita, opera bajo las mismas restricciones.

El benchmark inicial propuesto contiene 100 consultas curadas y 30 tareas técnicas. Se mantiene un conjunto de validación separado. Incluirá preguntas en español sobre documentación inglesa, símbolos exactos, nombres parecidos, versiones incompatibles, APIs eliminadas y casos sin respuesta documental suficiente.

Se comparan tres configuraciones sobre los mismos datos: búsqueda exacta/textual; búsqueda híbrida con pgvector; e híbrida con reordenamiento si la segunda deja fallos relevantes. Se mide recall de evidencias esperadas, ranking, exclusión de versiones incorrectas, latencia p95, coste por tarea resuelta y resultado técnico final.

La adopción inicial de embeddings exige, como objetivo propuesto, una mejora de al menos cinco puntos porcentuales de recall@10 sobre el conjunto reservado, sin introducir fallos en filtros obligatorios y dentro del presupuesto operativo acordado tras medir la base. Se repite el ensayo para evitar aceptar ruido de una muestra pequeña. Si la referencia ya resuelve el corpus, se conserva la solución más simple.

Los índices aproximados de pgvector pueden perder resultados al combinarse con filtros; se contrastan contra una búsqueda exacta de referencia y se evalúan particiones o búsquedas iterativas. La semejanza semántica no prueba la verdad de un fragmento. [S8]

No se mezclan embeddings de modelos distintos en el mismo espacio. El modelo, revisión, dimensiones y normalización forman parte de la identidad del índice. Una reconstrucción genera una versión nueva que se activa después de evaluarse.

### 7. Construir el sistema con su propia metodología

La versión inicial se desarrolla con un conjunto pequeño de reglas explícitas y verificadores convencionales. El proceso se registra manualmente donde aún no exista automatización. La automatización se incorpora después de demostrar el procedimiento.

Cada tarea sigue este ciclo: requisito y caso de fallo; fuentes de la versión elegida; decisión breve con alternativas; prueba que detecta el fallo; implementación mínima; verificaciones; revisión; evidencia conservada. Las pruebas relevantes al comportamiento se escriben antes o junto con la implementación, y se comprueba que detectan una implementación defectuosa.

La versión estable puede ayudar a implementar la candidata. La candidata se prueba en un entorno nuevo. Las pruebas protegidas y la firma de publicación quedan fuera de su control. Un cambio legítimo en la especificación o en una prueba requiere revisión explícita de ese cambio; no se admite modificar el oráculo únicamente para que la implementación pase.

Se emplean estrategias concretas de diseño: monolito modular, tipos para estados válidos, transacciones para consistencia, idempotencia para reintentos, adaptadores para integraciones y registros de decisiones para elecciones difíciles. Otras metodologías se adoptan cuando resuelven un problema observado.

### 8. Ejecución y uso cotidiano

Antes de comenzar, el sistema lee el commit y las políticas del proyecto, verifica la publicación y fija el contrato. Comprueba el entorno real mediante comandos y hashes. El mismo contrato se utiliza para documentación, ejecución y validación.

El usuario puede pedir «implementa esta funcionalidad siguiendo el contrato del proyecto». El coordinador selecciona pocos especialistas; cada trabajo tiene archivos o responsabilidades delimitados. Los cambios se realizan en un workspace aislado y producen un diff revisable.

El ejecutor recopila salidas reales, códigos de retorno y artefactos. No acepta como prueba una frase del modelo diciendo que los tests pasaron. El informe final reúne el diff, fuentes relevantes, comprobaciones, limitaciones concretas y desviaciones.

Las instalaciones de dependencias utilizan bloqueos congelados dentro del entorno declarado. Cuando falta una dependencia se crea una propuesta de modificación del contrato. Los servicios cloud se comprueban con pruebas de integración y versiones de API cuando existan; Nix no congela el comportamiento externo de un proveedor.

La integración local distribuye perfiles nativos, mientras que la integración remota puede solicitar tareas a un ejecutor común. Las suscripciones de las interfaces no se presuponen credenciales utilizables por ese backend; se configuran expresamente sus mecanismos de acceso al modelo.

Los procesos largos se representan como trabajos consultables y cancelables. Las operaciones externas usan claves de idempotencia cuando el servicio lo permite; tras un fallo incierto se reconcilia el estado antes de repetir.

### 9. Compatibilidad y autorización

El MCP implementa un conjunto reducido de herramientas con esquemas explícitos. Su compatibilidad se comprueba frente a las revisiones soportadas y en cada cliente. Las pruebas incluyen autenticación, descubrimiento, parámetros inválidos, expiración de credenciales, reconexiones, cancelación y errores de aplicación. [S12, S13]

La autenticación remota se apoya en un proveedor existente y en el perfil de autorización MCP. La aceptación exige ensayos de extremo a extremo con los clientes seleccionados; compatibilidad genérica OIDC no sustituye esas pruebas. Las credenciales se inyectan en ejecución y no se guardan en Git, paquetes de agentes ni artefactos públicos de Nix.

Los adaptadores generan archivos deterministas para versiones concretas de los clientes. Si una capacidad obligatoria no tiene equivalente, la generación falla con un informe de incompatibilidad. El resultado no afirma conservar semántica idéntica entre clientes sin probarla.

Las interfaces web pueden cambiar sin que el usuario fije su versión. En ellas se registra fecha, capacidades observadas y resultados de pruebas; el informe tiene vigencia acotada. La disponibilidad de la aplicación Desktop sobre NixOS es otra comprobación distinta de la compatibilidad del protocolo.

### 10. Actualizaciones y operación

La ingestión detecta cambios y crea snapshots candidatos. Los cambios en documentación y los cambios en instrucciones se evalúan por separado. Una actualización documental nunca sobrescribe las reglas activas directamente.

La promoción utiliza estados: descubierta, capturada, candidata, validada, aprobada, retirada. Un trabajo ya iniciado conserva su snapshot. El canal recomendado puede avanzar sin cambiar los contratos existentes.

Para nuevas publicaciones se ejecuta el mismo conjunto protegido, se comparan resultados con la anterior y se despliega primero en un entorno de prueba o un grupo limitado. Las políticas previamente autorizadas pueden promover automáticamente cambios que satisfagan todos sus criterios. Las modificaciones de permisos, gobernanza o compatibilidad requieren reglas específicas.

Los reintentos se limitan y los trabajos fallidos se conservan con causa. Un worker usa lease y heartbeat; perder el lease impide confirmar resultados obsoletos. Los procesos recuperados inspeccionan operaciones ya aplicadas antes de repetirlas.

Para sostener la operación se necesitan copias de la base y objetos, restauración ensayada, retención definida, métricas de cola, latencia, tasa de éxito, consumo de modelos y coste por tarea verificada. Se prueban falta de espacio, pérdida de conexión y reinicio de servicios. Una política de garbage collection conserva los objetos referenciados por publicaciones y contratos retenidos.

Objetivos provisionales para la primera instalación compartida: pérdida máxima de datos de una hora y restauración del servicio en cuatro horas. Son metas a demostrar mediante recuperación de base y objetos en una VM nueva; si no se cumplen, no se anuncian como garantías. Alta disponibilidad se decide a partir de una necesidad concreta, no se presupone.

## Matriz de verificación

| Capa | Prueba | Qué constituye evidencia |
|---|---|---|
| Contratos | Ejemplos y propiedades sobre IDs, versiones, rutas, dependencias y estados | Casos válidos aceptados; contradicciones y estados ilegales rechazados |
| Publicaciones | Firma real, clave equivocada, hash alterado, archivo extra, revocación | Verificador externo y binario coinciden en el rechazo/aceptación |
| Instalador | Repetición, conflicto local, fallo a mitad de escritura, recuperación | Ningún archivo ajeno modificado; recibo emitido sólo tras verificación completa |
| Rust | Formato, lints pertinentes, unitarias, integración y casos de documentación | Salidas y artefactos del código exacto que se publica [S14] |
| Parsers y límites | Property testing y fuzzing de manifiestos, rutas y respuestas | Invariantes respetadas y fallos reducidos a ejemplos reproducibles [S15, S16] |
| Base de datos | Migración desde versiones soportadas, concurrencia, permisos y restauración | Mismo estado esperado, sin filas indebidas ni trabajos perdidos |
| Worker | Duplicados, caída tras efecto externo, lease vencido y cancelación | Reconciliación e idempotencia; no se afirma ejecución exactamente una vez |
| NixOS | VM limpia, inicio de servicios, red, reinicio y actualización | Pruebas de sistema ejecutadas con la configuración publicada [S6] |
| MCP | Suite de conformidad y pruebas reales por cliente | Trazas de protocolo, resultado funcional y versión probada [S12, S13] |
| Recuperación | Consultas conocidas, bilingües, adversas y sin respuesta | Evidencia correcta, versión aplicable y comportamiento ante ausencia |
| Agentes | Tareas reales contra pruebas independientes | El artefacto satisface el contrato; trazas y coste registrados [S17] |
| Actualizador | Cambio documental inocuo, API incompatible y fuente maliciosa | Promoción correcta o bloqueo; snapshot anterior recuperable |

No se usa un porcentaje de cobertura de líneas como prueba suficiente de calidad. Para algoritmos puros se añaden pruebas por propiedades; para invariantes críticos se verifica que mutaciones deliberadas sean detectadas. El análisis formal se reserva para estados o algoritmos críticos acotados cuando aporte valor.

Los evaluadores de lenguaje pueden ayudar a revisar claridad o pertinencia arquitectónica. Las versiones, compilación, migraciones, permisos y resultados de pruebas se comprueban con mecanismos independientes. El conjunto protegido no se entrega como material de ajuste al agente que se evalúa.

El criterio mínimo de publicación exige cero fallos en invariantes obligatorios dentro de la batería declarada, todos los flujos críticos de aceptación superados y ausencia de regresiones bloqueantes frente a la anterior. Las tareas estocásticas se repiten y se informa la distribución de resultados y el tamaño de muestra; no se presenta una ejecución exitosa como garantía general.

## Plan de entregas

| Entrega | Resultado utilizable | Condición de salida |
|---|---|---|
| 1. Publicaciones verificables | CLI que valida un paquete exacto, su confianza y compatibilidad declarada | Batería de integridad, autenticidad, revocación y manipulación superada |
| 2. Distribución local | Instalador y adaptadores Codex/Claude Code con detección de cambios | Instalar dos veces, actualizar y recuperar un fallo conserva el estado previsto |
| 3. Contrato y entorno | Lectura de bloqueos y validación en NixOS | Detectar herramientas ajenas, versiones incorrectas y cambios no autorizados |
| 4. Servicio y documentación | PostgreSQL, MCP, snapshots y búsqueda textual | Fuentes trazables y herramientas consumibles por ambos clientes locales |
| 5. Ejecución y clientes remotos | Trabajos, evidencias, OAuth y conexión desde Claude/ChatGPT | Flujo completo autorizado y observable desde cada interfaz |
| 6. Recuperación evaluada | Benchmark y decisión de incorporar o descartar vectores | Mejora demostrada o decisión documentada de mantener la referencia |
| 7. Actualizaciones controladas | Candidatos, comparación, promoción y reversión | Actualización compatible promovida; incompatible bloqueada o migrada explícitamente |
| 8. Ampliación del catálogo | Nuevas tecnologías y perfiles de ingeniería | Cada especialista nuevo aporta casos propios y supera los compartidos |

Se comienza con un especialista Rust/Nix que ayude a construir la plataforma, otro de PostgreSQL y uno Nuxt/Vue/TypeScript como proyecto consumidor de referencia. Las funciones amplias se incorporan por tareas verificables, no por tamaño del catálogo.

## Primer hito: plan ejecutable del verificador local

### Contrato del hito

La CLI propuesta se llama `engctl`; todavía no existe. El comando previsto es `engctl verify --bundle <dir> --trust <file> --revocations <file> --target <id> --json`.

El paquete contiene `manifest.json`, `manifest.json.minisig` y los archivos enumerados por el manifiesto. La firma cubre los bytes originales del manifiesto. El ID de contenido es el SHA-256 de esos mismos bytes. No se incluye el hash del manifiesto dentro de sí mismo.

Esquema inicial: `schema_version = 1`, `release_id`, `supported_targets` y `files`. Cada entrada contiene `path`, `sha256`, `size_bytes` y `role`. Los roles admitidos son `instructions`, `skill`, `policy`, `source_spec` y `test_definition`. La lista de archivos se ordena por ruta al producir la publicación; verificar no reserializa el manifiesto firmado.

Límites iniciales propuestos: manifiesto de hasta 1 MiB, entre 1 y 1024 archivos, hasta 16 MiB por archivo y hasta 256 MiB de contenido total. Estos límites pertenecen a paquetes de instrucciones; los manuales completos se almacenan fuera del paquete. Las rutas son relativas UTF-8, con separador `/`, sin segmentos vacíos, `.` ni `..`, sin prefijo absoluto, NUL o `\\`. Se rechazan enlaces simbólicos en todos los segmentos y archivos adicionales distintos de los dos metadatos permitidos.

La verificación trabaja sobre una copia de staging no modificable por el productor mientras se inspecciona. Los recibos se consideran obsoletos si cambia el contenido; el ejecutor posterior vuelve a validar antes de usarlo. No se promete evitar carreras frente a un administrador hostil del sistema operativo.

Un registro de confianza contiene claves públicas y IDs autorizados; se distribuye mediante un canal previamente confiable. El registro de revocación tiene fecha de validez y también procede de ese canal. En este hito ambos son entradas locales suministradas por el operador; la ingestión remota futura debe verificar su autenticidad y evitar retrocesos de versión. El ejecutor administrado fija su configuración de confianza: un argumento enviado por un modelo no puede sustituirla. Estando ausente o vencido el registro de revocación, la CLI puede describir integridad, pero devuelve `authorization = unknown` y código 2. Una publicación revocada, denegada o cualquier validación fallida devuelve código 1. Sólo `authorization = approved` con todas las comprobaciones correctas devuelve 0.

El formato JSON de resultado incluye `schema_version`, `release_id`, `manifest_sha256`, `signer_id`, `target`, `integrity`, `authorization` y `errors`. Los errores tienen `code` y `path` opcional. Los tiempos de verificación se informan por separado; no forman parte de un digest esperado estable.

### Task 1: esquema e invariantes del manifiesto

**Files:** crear `Cargo.toml`, `rust-toolchain.toml`, `Cargo.lock`, `flake.nix`, `flake.lock`, `crates/contracts/Cargo.toml` (paquete `eng-contracts`), `crates/contracts/src/lib.rs`, `crates/contracts/src/manifest.rs`, `crates/contracts/tests/manifest.rs`, `docs/evidence/initial-toolchain.md` y `tests/fixtures/bundles/`.

**Interfaces:** producir `Manifest`, `FileEntry`, `FileRole`, `ValidationError`; `parse_manifest(bytes: &[u8]) -> Result<Manifest, ValidationError>`. Los campos corresponden exactamente al contrato del hito.

- [ ] Resolver y documentar una combinación de toolchain y bibliotecas soportada; fijar revisiones y archivos de bloqueo. El entorno incluye el verificador Minisign para las tareas siguientes.
- [ ] Escribir tests `accepts_valid_manifest`, `rejects_duplicate_paths`, `rejects_escape_paths`, `rejects_unknown_schema` y `enforces_size_limits`. Comprobar aceptación de los límites y rechazo al excederlos en una unidad; `files.len() == 1024` es válido si respeta los demás límites.
- [ ] Ejecutar `cargo test -p eng-contracts --locked` y comprobar que los tests de comportamiento fallan antes de implementar sus validaciones.
- [ ] Implementar el parser y las invariantes en `manifest.rs`; usar errores tipados y rechazar campos desconocidos para la versión 1.
- [ ] Repetir el comando hasta PASS; ejecutar formato y compilar en el entorno Nix fijado.
- [ ] Revisar la evidencia de versiones y registrar el cambio en Git junto con los bloqueos.

### Task 2: integridad y autenticidad reales

**Files:** crear `crates/catalog/Cargo.toml` (paquete `eng-catalog`), `crates/catalog/src/lib.rs`, `crates/catalog/src/verify.rs`, `crates/catalog/tests/integrity.rs`, `crates/catalog/tests/authenticity.rs`, `tests/fixtures/trust/` y paquetes firmados de prueba. Actualizar el workspace y su archivo de bloqueo.

**Interfaces:** consumir `Manifest`; producir `TrustStore`, `VerifiedBundle` y `VerificationError`; `verify_integrity(root: &std::path::Path, trust: &TrustStore) -> Result<VerifiedBundle, VerificationError>`. `VerifiedBundle` contiene manifiesto, hash y firmante comprobados. El catálogo invoca la implementación de Minisign fijada; no implementa criptografía propia.

- [ ] Crear fixtures reproducibles con claves exclusivas de tests: válida autorizada, válida no autorizada, firma corrupta y archivo alterado. Calcular hashes esperados con una herramienta independiente del código que se probará.
- [ ] Escribir `accepts_trusted_signature`, `rejects_valid_untrusted_signer`, `rejects_tampered_file`, `rejects_extra_file`, `rejects_symlink_component` y `rejects_size_mismatch`; cada rechazo comprueba su código de error específico.
- [ ] Ejecutar `cargo test -p eng-catalog --locked` y observar los fallos pertinentes.
- [ ] Implementar verificación de firma sobre bytes originales, enumeración exacta y hashes de archivos. Limitar recursos antes de cargar contenido y no ejecutar payloads.
- [ ] Repetir hasta PASS; comprobar que la firma de los fixtures también se valida o rechaza con el binario Minisign fuera de la biblioteca.
- [ ] Registrar en Git la implementación y fixtures; las claves de producción nunca participan en tests.

### Task 3: autorización, compatibilidad y recibo

**Files:** crear `crates/contracts/src/receipt.rs`, `crates/catalog/src/authorize.rs`, `crates/catalog/tests/authorization.rs` y `crates/catalog/tests/revalidation.rs`; actualizar los `lib.rs` de contratos y catálogo para exportar las interfaces.

**Interfaces:** producir `RevocationSnapshot`, `VerificationReceipt`, `AuthorizationStatus::{Approved, Denied, Revoked, Unknown}` y `IntegrityStatus::{Verified, Failed}`. `authorize(bundle: &VerifiedBundle, target: &str, revocations: Option<&RevocationSnapshot>, now: std::time::SystemTime) -> VerificationReceipt`. La hora es un argumento para probar vencimientos sin esperas.

- [ ] Escribir `rejects_revoked_release`, `marks_expired_revocations_unknown`, `rejects_unsupported_target` y `detects_changed_content_on_reverification`. El último altera un byte tras una verificación correcta y exige error de hash en la siguiente.
- [ ] Ejecutar los tests de `eng-catalog` y comprobar sus fallos iniciales.
- [ ] Implementar autorización separada de integridad. Un target no listado genera `unsupported_target` y `AuthorizationStatus::Denied`; se devuelve un recibo no utilizable, sin degradación silenciosa.
- [ ] Ejecutar hasta PASS, incluyendo el instante exacto de expiración y entradas de revocación duplicadas rechazadas durante su lectura.
- [ ] Registrar el cambio en Git después de revisar que la firma válida por sí sola no produce autorización.

### Task 4: CLI y ensayo de aceptación NixOS

**Files:** crear `apps/engctl/Cargo.toml` (paquete `engctl`), `apps/engctl/src/main.rs`, `apps/engctl/tests/verify_cli.rs`, `nix/tests/verifier.nix`, `docs/usage/verify.md` y `docs/acceptance/verifier.md`; actualizar workspace, flake y bloqueos.

**Interfaces:** consumir parser, `verify_integrity` y `authorize`; producir la CLI, su JSON versionado y códigos de salida 0, 1 y 2 definidos en el contrato. Para errores anteriores a leer un release válido, `release_id`, hash y firmante pueden ser `null`.

- [ ] Escribir aceptación del comando con paquete aprobado, alterado, revocado, firmante no confiable, target incompatible y registro de revocación vencido. Comprobar JSON, código de salida y ausencia de cambios en los archivos de entrada.
- [ ] Ejecutar `cargo test -p engctl --locked` y registrar los casos que fallan antes de conectar los componentes.
- [ ] Implementar CLI y serialización; stdout contiene únicamente JSON cuando se usa `--json`, mientras los diagnósticos auxiliares van a stderr.
- [ ] Ejecutar `cargo test --workspace --locked`, `cargo fmt --all -- --check` y los lints seleccionados para el workspace.
- [ ] Añadir un test NixOS con el paquete y sus dependencias declaradas que ejecute la misma aceptación desde una VM limpia. Exponerlo como `checks.x86_64-linux.verifier`.
- [ ] Ejecutar `nix flake check` en un runner con virtualización disponible. Conservar configuración, revisión, logs y resultados; si esa capacidad no existe, el hito queda pendiente de esa comprobación.
- [ ] Revisar el diff y la matriz de aceptación; registrar el cambio en Git y producir el informe del hito. No etiquetar el resto de la plataforma como implementado por haber completado este verificador.

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

Se comprobaron las correspondencias entre objetivos y entregas, la separación entre integridad y autorización, el uso de nombres de interfaces en el primer hito, las fuentes de las capacidades afirmadas y la distinción entre metas y resultados. La revisión es documental: las pruebas descritas todavía deben implementarse y ejecutarse.
