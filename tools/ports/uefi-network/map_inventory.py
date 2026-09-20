#!/usr/bin/env python3
"""Apply reviewed symbol dispositions to a fresh exact-pin network snapshot."""
import json
from pathlib import Path
import re
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2];RAW=ROOT/'source/contracts/uefi/raw'
path=RAW/'network-inventory.json';doc=json.loads(path.read_text())
schema=json.loads((HERE/'schema.json').read_text());records={r['name']:r for r in schema['records']}
raw='source/contracts/uefi/raw/network.omg';helper='source/libraries/uefi/network_helpers.omg';fixture='tools/ports/uefi-network/main.omg';probe='tools/ports/uefi-network/src/lib.rs'
net_methods={28:'ipv4_octets',34:'ipv4_from_octets',40:'ipv4_octets',46:'ipv4_from_octets',73:'ipv6_octets',79:'ipv6_from_octets',85:'ipv6_octets',91:'ipv6_from_octets',156:'ip_new_v4',168:'ip_new_v6',185:'ip_into_octets',206:'ip_default',212:'ip_from_octets',221:'ip_new_v4',227:'ip_new_v6',233:'ip_new_v4',239:'ip_new_v6',267:'mac_octets',274:'mac_ethernet',283:'mac_from_ethernet',292:'mac_ethernet',299:'mac_from_octets'}
net_tests={308:'TEST_IPV4',309:'TEST_IPV6',315:'test_ip_roundtrips',323:'test_ip_roundtrips',333:'test_ip_roundtrips',349:'test_mac_conversions',372:'test_promised_conversions',440:'test_flow_v4',441:'test_flow_v4',460:'test_flow_v4',486:'test_ip_roundtrips'}
http_defaults={58:'http_access_point_default',74:'http_token_default',94:'http_message_default',113:'http_request_data_default',157:'http_request_or_response_default',172:'http_header_default'}
pxe_methods={173:'pxe_server_new',185:'pxe_server_address',304:'pxe_packet_bytes',371:'pxe_bootp_ident',377:'pxe_bootp_seconds',383:'pxe_bootp_flags_upstream',389:'pxe_dhcp_magik',422:'pxe_transaction_id',443:'pxe_ip_filter_new',469:'pxe_ip_filter_address'}
def entry(disposition,reason,target=None,anchor=None):
 value={'disposition':disposition,'reason':reason}
 if target:value['targets']=[{'path':target,'anchor':anchor}]
 return value
for source,file in doc['files'].items():
 stem=Path(source).stem;lines=(ROOT/'reference_code/rust-osdev/uefi-rs'/source).read_text().splitlines()
 for key,symbol in file['symbols'].items():
  number=int(key.split(':')[0]);name=key.split(':')[1];a=symbol['anchor']
  previous='\n'.join(lines[:number]);decls=re.findall(r'pub (?:struct|enum|union) (\w+)',previous);owner=decls[-1] if decls else None
  if name=='fmt':value=entry('omitted','Rust formatting/Debug presentation has no raw firmware or pure protocol semantics; no Omega formatting adapter claimed.')
  elif stem=='net' and number in net_methods:
   value=entry('translated','Pure initialized-byte behavior; Rust core::net types replaced by fixed octets/IpOctets, with no raw pointer or unchecked union access.',helper,'machine '+net_methods[number]+'(')
  elif stem=='net' and number in net_tests:
   value=entry('translated','Behavioral fixture uses equivalent octets; mock firmware flow is an owned initialized result, not a foreign call or aliasing test.',fixture,net_tests[number])
  elif stem=='net' and name in {'_','PackedHelper'}:
   value=entry('translated','Compile-time representation expectations independently asserted by the UEFI-target Rust probe; not an Omega ABI result.',probe,'uefi_raw::IpAddress')
  elif stem=='http' and number in http_defaults:
   value=entry('translated','Zero/null default translated as initialized ordinary data; raw addresses confer no access.',helper,'machine '+http_defaults[number]+'(')
  elif stem=='pxe' and number in pxe_methods:
   value=entry('translated','Pure behavior translated; optional addresses are owned snapshots, filters use fixed capacity/count and checked optional indexing; bootp_flags preserves pin explicitly.',helper,'machine '+pxe_methods[number]+'(')
  elif stem=='pxe' and number in {312,320}:
   value=entry('blocked','PORT-BLOCKED[omega:plan-laid-public-fields]: borrowed packet projection via checked byte-region recast reaches private synthesized raw-field diagnostic; see layout_union_view.omg. No unchecked reference substitute.',helper,'PORT-BLOCKED[omega:plan-laid-public-fields]')
  elif stem=='snp' and 'fn ' in a:
   target='network_statistic_available' if name=='available' else 'network_statistic' if name=='to_option' else 'network_statistic_'+name
   value=entry('translated','Preserves all-ones unavailable sentinel; owned nominal optional replaces Rust Option.',helper,'machine '+target+'(')
  elif name in records and ('struct ' in a or 'enum ' in a or 'union ' in a or 'type ' in a):
   value=entry('translated','Explicit inert raw x64 representation; see schema, foreign plans, and deliberate deviations.',raw,'pub data '+name+' [copy]')
  elif re.match(r'pub\s+\w+\s*:',a):
   field=next((f for f in records.get(owner,{}).get('fields',[]) if f[0]==name),None)
   if field is None:raise ValueError((source,key,owner,'unknown field'))
   typ=field[1]
   if re.search(r';\s*0\s*\]',typ):value=entry('blocked','PORT-BLOCKED[omega:runtime-layout-strides]: bounded flexible tail must bind runtime extent/stride; fixed prefix and expected tail offset retained.',raw,name+': '+typ+' needs a bounded runtime tail view.')
   elif records[owner]['kind']=='union':value=entry('translated','Union carrier bytes/address retained with every alternative and offset explicit; does not claim a typed overlay or reference authority.',raw,'Overlapping Rust member '+name+': '+typ)
   else:value=entry('translated','Field order/type retained; pointer/function slots are inert addr with full pinned signature adjacent.',raw,name+': '+dict(records[owner]['omega_fields'])[name]+';')
  elif name=='ZERO':value=entry('translated','Explicit all-16-byte initialized zero address.',raw,'pub const IP_ADDRESS_ZERO:')
  elif name in {'tests','dhcp4','http','ip4','ip4_config2','pxe','snp','tcp4','tls'} and 'mod ' in a:
   value=entry('omitted','Rust module/test namespace flattened; corresponding declarations and tests have individual mappings.')
  else:
   matches=[c for c in schema['constants'] if c['owner']==owner and c['rust'].endswith('::'+name)]
   if len(matches)!=1:raise ValueError((source,key,a,owner,'unmapped'))
   value=entry('translated','Pinned discriminant/bitmask/GUID preserved exactly and cross-target verified.',raw,'pub const '+matches[0]['name']+':')
  symbol.update(value)
 file['disposition']='omitted' if not file['symbols'] else 'blocked' if any(v['disposition']=='blocked' for v in file['symbols'].values()) else 'translated'
 file['reason']='Complete lexical review; per-symbol mappings distinguish fixed raw carriers, pure behavior, presentation omissions and exact blocked tails/views.'
 if file['disposition']=='translated':file['targets']=[{'path':raw,'anchor':'module network;'}]
path.write_text(json.dumps(doc,indent=2)+'\n')
from collections import Counter
print(Counter(v['disposition'] for f in doc['files'].values() for v in f['symbols'].values()))
