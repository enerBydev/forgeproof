import test from 'node:test';
import assert from 'node:assert/strict';
import { inspectReceipt } from '../public/receipt.mjs';

const valid = () => ({schema_version:1,release_id:'rust-nix@0.1.0',manifest_sha256:'b'.repeat(64),signer_id:'publisher',target:'codex',integrity:'verified',authorization:'approved',errors:[]});

test('an imported approved receipt never becomes authenticated evidence', () => {
  const result=inspectReceipt(JSON.stringify(valid()));
  assert.equal(result.ok,true);
  assert.equal(result.authenticated,false);
  assert.equal(result.label,'Aprobación declarada');
});
test('rejects approval with failed integrity', () => {
  assert.equal(inspectReceipt(JSON.stringify({...valid(),integrity:'failed'})).ok,false);
});
test('rejects approval with errors or missing identifiers', () => {
  for(const patch of [{errors:[{code:'hash_mismatch'}]}, {release_id:null},{signer_id:null},{manifest_sha256:null}]) {
    assert.equal(inspectReceipt(JSON.stringify({...valid(),...patch})).ok,false);
  }
});
test('unknown authorization stays unknown', () => {
  const result=inspectReceipt(JSON.stringify({...valid(),authorization:'unknown',errors:[{code:'revocations_expired'}]}));
  assert.equal(result.ok,true); assert.equal(result.label,'Vigencia desconocida');
});
test('accepts failed receipt without known bundle identity', () => {
  assert.equal(inspectReceipt(JSON.stringify({...valid(),release_id:null,manifest_sha256:null,signer_id:null,integrity:'failed',authorization:'denied',errors:[{code:'invalid_signature'}]})).ok,true);
});
test('rejects malformed, oversized and unsupported data', () => {
  for(const text of ['{', 'null', '[]',' '.repeat(65537),JSON.stringify({...valid(),schema_version:2}),JSON.stringify({...valid(),manifest_sha256:'bad'}),JSON.stringify({...valid(),authorization:'magic'}),JSON.stringify({...valid(),extra:true})]) {
    assert.equal(inspectReceipt(text).ok,false);
  }
});
test('rejects attempts to embed markup in public identifiers', () => {
  assert.equal(inspectReceipt(JSON.stringify({...valid(),release_id:'<img src=x onerror=alert(1)>'})).ok,false);
});
test('revocation is not displayed as approval', () => {
  const result=inspectReceipt(JSON.stringify({...valid(),authorization:'revoked',errors:[{code:'release_revoked'}]}));
  assert.equal(result.ok,true); assert.equal(result.label,'Publicación revocada');
});
test('rejects authorization values that coerce to an allowed state', () => {
  for (const authorization of [['approved'], ['unknown'], {toString:'approved'}, {toString:null}, null, 0]) {
    assert.equal(inspectReceipt(JSON.stringify({...valid(),authorization,errors:[{code:'revocations_expired'}]})).ok,false);
  }
});
