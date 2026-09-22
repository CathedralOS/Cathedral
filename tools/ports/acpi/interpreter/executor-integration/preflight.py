#!/usr/bin/env python3
"""Audit source ownership and render the whole corpus without executing Omega."""
import argparse
import ast
import json
import re
import check
import provenance


def run():
    before=check.snapshot()
    source_audit=provenance.run()
    for group,filename in [('mixed','mixed-cases.json'),('frame','frame-cases.json'),('composed_writes','pipeline-cases.json')]:
        assert check.rows(group)==json.loads((check.HERE/filename).read_text()),'stale '+filename
    for path in check.HERE.glob('*.py'):
        ast.parse(path.read_text(),filename=str(path))
    groups=[]
    for group in check.GROUPS:
        rows=check.rows(group)
        assert len(rows)==len({row['name'] for row in rows})
        source,entries=check.module_source(group,rows)
        assert len(entries)==len(set(entries))==2*len(rows)
        assert not re.search(r'(?<![A-Za-z0-9_])18446744073709551616(?![A-Za-z0-9_])',source)
        for index,row in enumerate(rows):
            assert entries[index*2].endswith('::'+row['name']+'_positive=0')
            assert entries[index*2+1].endswith('::'+row['name']+'_control=1')
        selections=[]
        for index in sorted({0,len(rows)//2,len(rows)-1}):
            subset,names=check.module_source(group,[rows[index]])
            assert names==entries[index*2:index*2+2]
            selections.append(dict(case=rows[index]['name'],source_sha256=check.text_sha(subset),entries=names))
        groups.append(dict(group=group,count=len(rows),source_sha256=check.text_sha(source),
                           cases=[row['name'] for row in rows],selections=entries,sampled_subsets=selections))
    frame=check.fixtures.frames()
    original,_=check.fixtures.original_source('logical_bridge',check.rows('logical_bridge')[:1])
    augmented=frame.augment(original)
    assert augmented!=original and frame.augment(augmented)==augmented
    rejected=[]
    for label,old,new in [
        ('missing_mode','a.field_writes==b.field_writes && ',''),
        ('missing_inactive_value','value==other_value && ',''),
        ('duplicate_import',frame.IMPORT,frame.IMPORT+'\n'+frame.IMPORT),
    ]:
        assert old in augmented
        changed=augmented.replace(old,new,1)
        try:
            frame.augment(changed)
        except ValueError:
            rejected.append(label)
        else:
            raise AssertionError('partial comparator accepted: '+label)
    assert before==check.snapshot()
    count=sum(group['count'] for group in groups)
    assert count==1091
    return dict(stage='source provenance and host rendering only',execution_validation=False,
                positive_count=count,control_count=count,source_unchanged=True,input_sha256=before,
                source_audit=source_audit,groups=groups,rejected_partial_comparators=rejected)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--record',type=check.Path)
    args=parser.parse_args()
    record=run()
    if args.record:
        args.record.write_text(json.dumps(record,indent=2)+'\n')
    print('PASS host generation:',record['positive_count'],'pairs;',len(record['input_sha256']),
          'bound inputs; no Omega execution claim')


if __name__=='__main__':
    main()
