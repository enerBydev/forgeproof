import { cpSync, mkdirSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
mkdirSync('dist',{recursive:true});
cpSync('apps/web/public','dist',{recursive:true});
let revision=process.env.VERCEL_GIT_COMMIT_SHA??'untracked';
if(revision==='untracked') {try{revision=execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim();}catch{}}
writeFileSync('dist/status.json',JSON.stringify({name:'forgeproof',schema_version:1,revision,capability:'receipt-inspection',authenticated_verification:false},null,2)+'\n');
console.log(`Built Forgeproof receipt inspector (${revision}).`);
