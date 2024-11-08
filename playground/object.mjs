import { DicomFile, saveRawPixelData } from '../index.js';

const file = new DicomFile();

file.open('./__test__/fixtures/test.dcm');

file.saveRawPixelData('./tmp/raw_pixel_data.jpg');

saveRawPixelData('./__test__/fixtures/test.dcm', './tmp/raw_pixel_data_2.jpg');

console.log(file.getElements());

file.close();