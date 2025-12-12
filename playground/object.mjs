import { DicomFile, createCustomTag } from '../index.js';

(async () => {
    const file = new DicomFile();

    await file.open('./__test__/fixtures/8B1FA77C.dcm');

    const data = file.extract(['AcquisitionDate', 'Modality', 'PatientName'], [createCustomTag('00091001', 'VendorPrivateTag')]);
    console.log('Extracted tags:', data);

    await file.saveRawPixelData('./tmp/raw_pixel_data_2.txt');

    console.log(file.getElements());

    file.close();
})();