#!/usr/bin/env python3
"""Host-only packing and receipt-rejection tests; synthetic outputs are not execution evidence."""
import copy
import json
from pathlib import Path
import unittest
import check
import verify_record


def synthetic_record(selections, batch_size=2000, combine=True, scope='selected'):
    """Construct test data in memory only, never a checked-verification artifact."""
    batches = []
    for plan in check.plan_batches(selections, batch_size, combine):
        _, modules, entries, driver = check.render_package(plan)
        output = 'CHECKED authored package and dependency bodies; native publication NOT requested\n'
        for entry in entries:
            name, expected = entry.rsplit('=', 1)
            output += 'PASS '+name+' expected='+expected+' observed='+expected+' error=None usage=HOST_TEST_ONLY\n'
        batches.append(dict(modules=modules, driver_sha256=check.text_sha(driver), selections=entries,
                            exit_code=0, source_unchanged=True, elapsed_seconds=0, output=output))
    count = sum(len(item['cases']) for item in selections)
    build = check.build_text()
    return dict(format_version=2, stage='SYNTHETIC HOST TEST ONLY; not Omega execution evidence',
                omega_revision=check.PIN, exit_code=0, source_unchanged=True, input_sha256=check.snapshot(),
                runner_path=str(check.RUNNER), runner_sha256=check.sha(check.RUNNER), scope=scope,
                groups=[item['group'] for item in selections], groups_selected=selections,
                batch_size=batch_size, workers=1, combine_groups=combine, batches=batches,
                positive_count=count, control_count=count, batch_elapsed_seconds_sum=0,
                execution_root=str(check.ROOT), build_source=build, build_sha256=check.text_sha(build))


class PackingTests(unittest.TestCase):
    def test_cross_group_boundary(self):
        selected = [dict(group='mixed', cases=['a', 'b', 'c']),
                    dict(group='frame', cases=['d', 'e', 'f', 'g']),
                    dict(group='logical', cases=['h', 'i'])]
        self.assertEqual(check.plan_batches(selected, 4, True), [
            [dict(group='mixed', cases=['a', 'b', 'c']), dict(group='frame', cases=['d'])],
            [dict(group='frame', cases=['e', 'f', 'g']), dict(group='logical', cases=['h'])],
            [dict(group='logical', cases=['i'])],
        ])
        self.assertEqual(check.plan_batches(selected, 4, False), [
            [dict(group='mixed', cases=['a', 'b', 'c'])],
            [dict(group='frame', cases=['d', 'e', 'f', 'g'])],
            [dict(group='logical', cases=['h', 'i'])],
        ])

    def test_exact_boundary_and_large_group(self):
        selected = [dict(group='mixed', cases=list('abcdef')), dict(group='frame', cases=['g'])]
        self.assertEqual(check.plan_batches(selected, 3, True), [
            [dict(group='mixed', cases=list('abc'))],
            [dict(group='mixed', cases=list('def'))],
            [dict(group='frame', cases=['g'])],
        ])
        with self.assertRaises(AssertionError):
            check.plan_batches(selected, 0, True)
        with self.assertRaises(AssertionError):
            check.plan_batches(selected + [selected[0]], 3, True)

    def test_complete_corpus_retains_frozen_bodies(self):
        baseline = json.loads((check.HERE/'diagnostics/package-batching/before-render-audit.json').read_text())
        selected = [dict(group=group, cases=[row['name'] for row in check.rows(group)]) for group in check.GROUPS]
        plans = check.plan_batches(selected, 2000, True)
        self.assertEqual(len(plans), 1)
        sources, modules, entries, driver = check.render_package(plans[0])
        self.assertEqual(len(modules), 16)
        self.assertEqual(len(entries), 2182)
        self.assertEqual(list(sources), ['authored_'+group+'.omg' for group in check.GROUPS])
        for old, new in zip(baseline['groups'], modules):
            self.assertEqual(new, {key: old[key] for key in ('group', 'cases', 'source_sha256', 'selections')})
        self.assertEqual(driver, ''.join('use authored_'+group+';\n' for group in check.GROUPS))
        record = synthetic_record(selected, scope='full')
        self.assertEqual(verify_record.verify(record, require_binaries=True, require_complete=True), 1091)


class ReceiptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.selected = [dict(group=group, cases=[row['name'] for row in check.rows(group)[:2]])
                        for group in ('mixed', 'frame', 'composed_writes')]
        cls.record = synthetic_record(cls.selected)

    def test_both_package_modes(self):
        self.assertEqual(verify_record.verify(self.record, require_binaries=True), 6)
        separated = synthetic_record(self.selected, batch_size=1, combine=False)
        self.assertEqual(verify_record.verify(separated), 6)
        with self.assertRaises(AssertionError):
            verify_record.verify(self.record, require_complete=True)

    def test_missing_group_is_not_complete_even_with_consistent_counts(self):
        narrowed = synthetic_record([dict(group='frame', cases=[row['name'] for row in check.rows('frame')])], scope='full')
        self.assertEqual(verify_record.verify(narrowed), 32)
        with self.assertRaises(AssertionError):
            verify_record.verify(narrowed, require_complete=True)

    def test_receipt_mutations_are_rejected(self):
        def change(label, mutate):
            record = copy.deepcopy(self.record)
            mutate(record)
            with self.subTest(label=label), self.assertRaises(AssertionError):
                verify_record.verify(record)

        change('missing module', lambda r: r['batches'][0]['modules'].pop())
        change('reordered modules', lambda r: r['batches'][0]['modules'].reverse())
        change('wrong module body', lambda r: r['batches'][0]['modules'][0].update(source_sha256='0'*64))
        change('omitted control', lambda r: r['batches'][0]['selections'].pop())
        change('different driver', lambda r: r['batches'][0].update(driver_sha256='0'*64))
        change('wrong pinned runner', lambda r: r.update(runner_sha256='0'*64))
        change('changed dependency', lambda r: r['input_sha256'].update({'source/libraries/acpi/aml/model.omg': '0'*64}))
        change('changed build', lambda r: r.update(build_source=r['build_source']+' '))
        change('failed compile', lambda r: r['batches'][0].update(exit_code=2))
        change('failed run', lambda r: r.update(exit_code=1))
        change('changed source during run', lambda r: r['batches'][0].update(source_unchanged=False))
        change('wrong pair count', lambda r: r.update(control_count=5))
        change('wrong packing mode', lambda r: r.update(combine_groups=False))
        change('reordered cases', lambda r: r['groups_selected'][0]['cases'].reverse())
        change('missing observed control', lambda r: r['batches'][0].update(output='\n'.join(r['batches'][0]['output'].splitlines()[:-1])+'\n'))
        change('duplicate check marker', lambda r: r['batches'][0].update(output=r['batches'][0]['output'].splitlines()[0]+'\n'+r['batches'][0]['output']))
        change('extra failed entry', lambda r: r['batches'][0].update(output=r['batches'][0]['output']+'FAIL extra expected=0 observed=1 error=None usage=HOST_TEST_ONLY\n'))
        change('malformed extra PASS', lambda r: r['batches'][0].update(output=r['batches'][0]['output']+'PASS extra expected=0 observed=0 error=SomeError usage=HOST_TEST_ONLY\n'))
        change('incorrect observation', lambda r: r['batches'][0].update(output=r['batches'][0]['output'].replace('observed=0', 'observed=1', 1)))


if __name__ == '__main__':
    unittest.main()
