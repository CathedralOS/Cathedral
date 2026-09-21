import hashlib,json,re
from pathlib import Path
HERE=Path(__file__).resolve().parent
BASE=Path('/tmp/cathedral-field-read-decoder-micro')
ROOT=Path('/tmp/cathedral-field-reads-current')
sha=lambda b:hashlib.sha256(b).hexdigest()
def extract(raw,name):
    match=re.search(r'^(?:pub )?machine '+re.escape(name)+r'\(',raw,re.M);assert match,name
    start=raw.index('{',match.end());depth=1;end=start+1
    while depth:
        depth+=(raw[end]=='{')-(raw[end]=='}');end+=1
    return raw[match.start():end]
prior=json.loads((BASE/'manifest.json').read_text())
source=(BASE/'main.omg').read_text()
assert sha(source.encode())==prior['main_sha256'] and prior['exit_code']==0
original_stub=extract(source,'execution_turn')
copies=[];sources=dict(prior['sources']);substitutions=[]
def copied(relative,name,replacements=()):
    path=ROOT/relative;raw=path.read_text();sources[str(path)]=sha(path.read_bytes())
    original=extract(raw,name);body=original
    for a,b in replacements:
        assert a in body;body=body.replace(a,b)
    copies.append(dict(source=str(path),machine=name,original_sha256=sha(original.encode()),copied_sha256=sha(body.encode()),substitutions=list(replacements)))
    return body+'\n'
source=source.replace(original_stub,copied('source/libraries/acpi/interpreter/execution/engine.omg','execution_turn'))
for name in ('current_operation','pop_operation','push_operation'):
    source+=copied('source/libraries/acpi/interpreter/execution/operands.omg',name)
source+=copied('source/libraries/acpi/interpreter/execution/decode_execution.omg','named_operand',[('aml::model::Span','Span')])
names_path=ROOT/'source/libraries/acpi/interpreter/execution/execution_names.omg'
sources[str(names_path)]=sha(names_path.read_bytes())
declaration=re.search(r'^pub data NamedResult[^\n]+',names_path.read_text(),re.M).group(0)
source+=declaration+'\n'
source+='''
// Only the actual term-to-named-operand seam is substituted: deterministic
// name lookup reports the fixture's real Field slot and next source cursor.
machine decode_term(input:&[u8;1024],store:&ObjectStore,frame:&mut Frame)->ExecutionOutcome {
 let found:NamedResult=NamedResult {object_id:1,next:38};
 let value:Value=store.space.objects[1].value;
 let result:ExecutionOutcome=named_operand(input,store,frame,found,value);result
}
// These unrelated dispatch paths fail if reached by any diagnostic selection.
machine complete_frame(runtime:&mut Runtime,index:u64)->ExecutionOutcome {ExecutionOutcome::UnsupportedValue}
machine finish_block(frame:&mut Frame)->ExecutionOutcome {ExecutionOutcome::UnsupportedValue}
machine decode_target(input:&[u8;1024],store:&ObjectStore,frame:&mut Frame)->ExecutionOutcome {ExecutionOutcome::UnsupportedValue}
machine invoke_frame(runtime:&mut Runtime,definitions:&[MethodDefinition;64],index:u64,operation:Operation)->ExecutionOutcome {ExecutionOutcome::UnsupportedValue}
machine retire_operation(input:&[u8;1024],store:&mut ObjectStore,frame:&mut Frame,operation:Operation)->ExecutionOutcome {ExecutionOutcome::UnsupportedValue}
'''
build='machine build(builder:&mut Build){builder.package("field-read-dispatch-micro");builder.freestanding=true;}\n'
(HERE/'main.omg').write_text(source);(HERE/'build.omg').write_text(build)
manifest=dict(kind='Incremental real execution_turn and named_operand diagnostic; deterministic field-name decode; unrelated dispatch/validation/contribution fail if reached; flattened declarations, no loader/provider proof.',sources=sources,copies=prior['copies']+copies,base_manifest_sha256=sha((BASE/'manifest.json').read_bytes()),base_manifest=str(BASE/'manifest.json'),main_sha256=sha(source.encode()),build_sha256=sha(build.encode()),generator_sha256=sha(Path(__file__).read_bytes()),selections=prior['selections'],stubs=['decode_term supplies real fixture Field object1 and cursor38 to exact named_operand','validate_operand/contribute_operand/complete_frame/finish_block/decode_target/invoke_frame/retire_operation fail with UnsupportedValue'],substitutions=[['synthetic execution_turn','exact original execution_turn'],['aml::model::Span','Span in copied named_operand only']],named_result_declaration_sha256=sha(declaration.encode()))
(HERE/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Generated incremental real-dispatch diagnostic;',len(copies),'additional machine copies')
