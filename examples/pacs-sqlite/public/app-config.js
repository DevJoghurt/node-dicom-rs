/**
 * OHIF Viewer Configuration
 * 
 * Configure the viewer to use our local DICOMweb endpoints
 */

window.config = {
  routerBasename: '/',
  showStudyList: true,
  
  dataSources: [
    {
      namespace: '@ohif/extension-default.dataSourcesModule.dicomweb',
      sourceName: 'local-pacs',
      configuration: {
        friendlyName: 'Local PACS (SQLite)',
        name: 'local-pacs',
        wadoUriRoot: 'http://localhost:8043/dicomweb',
        qidoRoot: 'http://localhost:8042/dicomweb',
        wadoRoot: 'http://localhost:8043/dicomweb',
        qidoSupportsIncludeField: false,
        supportsReject: false,
        imageRendering: 'wadors',
        thumbnailRendering: 'wadors',
        enableStudyLazyLoad: true,
        supportsFuzzyMatching: false,
        supportsWildcard: false,
        staticWado: false,
        singlepart: 'bulkdata,video',
        bulkDataURI: {
          enabled: true,
        },
      },
    },
  ],
  
  defaultDataSourceName: 'local-pacs',
};
