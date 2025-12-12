import { DicomFile, saveRawPixelData, createCustomTag } from '../index.js';

const file = new DicomFile();

file.open('./__test__/fixtures/8B1FA77C.dcm');

file.extract(['AcquisitionDate', 'Modality', 'PatientName', ''], createCustomTag('00091001', 'VendorPrivateTag'), 'ByScope');

file.saveRawPixelData('./tmp/raw_pixel_data_2.txt');

saveRawPixelData('./__test__/fixtures/test.dcm', './tmp/raw_pixel_data_2.jpg');

console.log(file.getElements());

file.close();