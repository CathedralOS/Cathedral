import hashlib,json,os,subprocess,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
RUNNER=Path('/tmp/cathedral-field-protocol-target/release/cathedral-acpi-checked-runner')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
r=json.loads((HERE/'manifest.json').read_text())
r['runner']=str(RUNNER);r['runner_sha256']=sha(RUNNER)
r['run_script_sha256']=sha(Path(__file__))
started=time.monotonic()
command=[str(RUNNER),str(HERE/'main.omg'),str(HERE/'build'),*r['selections']]
r['command']=command
result=subprocess.run(command,capture_output=True,text=True,env=dict(os.environ,OMEGA_INTERP_STEP_BUDGET='10000000'))
output=result.stdout+result.stderr
(HERE/'output.log').write_text(output)
r.update(exit_code=result.returncode,elapsed_seconds=time.monotonic()-started,output=output,output_sha256=hashlib.sha256(output.encode()).hexdigest())
r['inputs_unchanged']=all(sha(Path(p))==want for p,want in r['sources'].items()) and sha(HERE/'main.omg')==r['main_sha256'] and sha(HERE/'build.omg')==r['build_sha256'] and sha(RUNNER)==r['runner_sha256']
(HERE/'manifest.json').write_text(json.dumps(r,indent=2)+'\n')
print(output,end='');print('inputs unchanged:',r['inputs_unchanged'],'elapsed:',round(r['elapsed_seconds'],3),flush=True)
raise SystemExit(result.returncode)
