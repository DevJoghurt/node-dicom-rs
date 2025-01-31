import test from 'ava'

import { DicomFile } from './../index'

test('read dicom file', (t) => {
  const file = new DicomFile();

  file.open('./__test__/fixtures/test.dcm');

  const data = file.getElements();

  t.is(data.patientName, 'CompressedSamples^CT1');
})
