"""Launch metadata stays byte-compatible and cannot silently widen requests."""
import copy
import struct
import unittest
from manifest import encode


class ManifestTests(unittest.TestCase):
    def setUp(self):
        self.value = {'imports': [dict(contract=1, minimum=1, maximum=1, required=True)],
                      'channels': 2, 'descendants': 0}

    def test_funded_interaction_record(self):
        record = encode(self.value)
        self.assertEqual(record[:8], b'CTHIMP03')
        self.assertEqual(struct.unpack('<17Q', record[8:]), (0x20001, 1, 1, 1, 1, *([0] * 12)))

    def test_unknown_optional_contract_is_preserved(self):
        self.value['imports'][0] = dict(contract=2**64-1, minimum=3, maximum=5, required=False)
        self.assertEqual(struct.unpack('<18Q', encode(self.value))[2:6], (2**64-1, 3, 5, 0))

    def test_reject_invalid_or_ambiguous_metadata(self):
        invalid = [dict(self.value, channels=True), dict(self.value, channels=65536),
                   dict(self.value, descendants=-1), dict(self.value, extra=1),
                   dict(self.value, imports=self.value['imports'] * 2)]
        for key, value in [('contract', 0), ('minimum', 0), ('maximum', 0), ('required', 1)]:
            item = copy.deepcopy(self.value)
            item['imports'][0][key] = value
            invalid.append(item)
        for item in invalid:
            with self.subTest(item=item), self.assertRaises(ValueError):
                encode(item)


if __name__ == '__main__':
    unittest.main()
