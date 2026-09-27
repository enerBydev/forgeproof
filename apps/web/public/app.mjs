import { inspectReceipt } from './receipt.mjs';

const $=id=>document.getElementById(id);
const form=$('receipt-form'), text=$('receipt-text'), file=$('receipt-file');
let source='PEGADO';
function showError(message) {
  $('receipt-result').hidden=true; $('empty-state').hidden=true;
  $('error-state').hidden=false; $('error-state').textContent=message;
}
function inspect() {
  const result=inspectReceipt(text.value);
  if(!result.ok) { showError(result.error); return; }
  $('error-state').hidden=true; $('empty-state').hidden=true; $('receipt-result').hidden=false;
  $('source-label').textContent=source;
  const r=result.receipt;
  $('decision').dataset.state=r.authorization;
  $('decision-label').textContent=result.label;
  $('integrity-label').textContent=r.integrity==='verified'?'El recibo declara integridad verificada.':'El recibo declara un fallo de integridad.';
  for(const [id,value] of [['release-value',r.release_id],['target-value',r.target],['signer-value',r.signer_id],['hash-value',r.manifest_sha256]]) $(id).textContent=value??'No determinada';
  $('errors-list').replaceChildren();
  for(const error of r.errors) { const li=document.createElement('li'); li.textContent=error.code+(error.path?' · '+error.path:''); $('errors-list').append(li); }
  $('issues').hidden=r.errors.length===0;
}
form.addEventListener('submit',event=>{event.preventDefault();inspect();});
text.addEventListener('input',()=>{source='PEGADO';$('source-label').textContent='ENTRADA MODIFICADA';$('receipt-result').hidden=true;$('error-state').hidden=true;$('empty-state').hidden=false;});
file.addEventListener('change',async()=>{
  const chosen=file.files[0]; if(!chosen)return;
  if(chosen.size>65536){showError('El recibo supera el límite de 64 KiB.');return;}
  try {text.value=await chosen.text();source='ARCHIVO LOCAL';inspect();} catch{showError('No se pudo leer el archivo.');}
});
$('load-example').addEventListener('click',()=>{
  source='EJEMPLO · DATOS FICTICIOS';
  text.value=JSON.stringify({schema_version:1,release_id:'demo-rust@0.1.0',manifest_sha256:'b'.repeat(64),signer_id:'demo-publisher',target:'codex',integrity:'verified',authorization:'unknown',errors:[{code:'revocations_missing'}]},null,2);
  file.value='';inspect();
});
