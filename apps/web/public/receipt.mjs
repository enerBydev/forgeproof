const id = value => typeof value === 'string' && /^[A-Za-z0-9._@-]{1,128}$/.test(value);
const hash = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
const record = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const fields = ['schema_version','release_id','manifest_sha256','signer_id','target','integrity','authorization','errors'];
const labels = {approved:'Aprobación declarada',denied:'Uso denegado',revoked:'Publicación revocada',unknown:'Vigencia desconocida'};

export function inspectReceipt(text) {
  const invalid = {ok:false,error:'El archivo no es un recibo Forgeproof v1 válido o contiene estados contradictorios.'};
  if(typeof text !== 'string' || new TextEncoder().encode(text).length > 65536) return invalid;
  let receipt;
  try { receipt=JSON.parse(text); } catch { return invalid; }
  if(!record(receipt) || Object.keys(receipt).length !== fields.length || !fields.every(key=>Object.hasOwn(receipt,key))) return invalid;
  const r=receipt;
  if(r.schema_version!==1 || !id(r.target) || !['verified','failed'].includes(r.integrity) || typeof r.authorization!=='string' || !Object.hasOwn(labels,r.authorization)) return invalid;
  if(!(r.release_id===null || id(r.release_id)) || !(r.signer_id===null || id(r.signer_id)) || !(r.manifest_sha256===null || hash(r.manifest_sha256))) return invalid;
  if(!Array.isArray(r.errors) || r.errors.length>128 || !r.errors.every(error=>record(error) && id(error.code) && Object.keys(error).every(k=>['code','path'].includes(k)) && (!Object.hasOwn(error,'path') || (typeof error.path==='string' && error.path.length<=4096)))) return invalid;
  if(r.integrity==='verified' && (!id(r.release_id) || !id(r.signer_id) || !hash(r.manifest_sha256))) return invalid;
  if(r.authorization==='approved' && (r.integrity!=='verified' || r.errors.length!==0)) return invalid;
  if(r.integrity==='failed' && (r.authorization!=='denied' || r.errors.length===0)) return invalid;
  return {ok:true,receipt:r,authenticated:false,label:labels[r.authorization]};
}
