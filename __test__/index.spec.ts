import test from 'ava'

import { DicomFile } from './../index'

test('read dicom file', async (t) => {
  const file = new DicomFile();

  await file.open('./__test__/fixtures/test.dcm');

  const data = file.extract(['PatientName']);

  t.is(data.PatientName, 'CompressedSamples^CT1');
})
