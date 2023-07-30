const { LanguageClient, TransportKind } = require('vscode-languageclient');

/*
In order to run this, you must open the `vscode` folder in vscode. It should autodetect it's trying to be an extension,
and give you the option to run it in the run & debug menu on the side.
 */

function activate (context) {
	const serverOptions = {
		// NOTE might need to edit this to point to your path
		command: 'rusty-pine',
		transport: TransportKind.icp
	};

	const clientOptions = {
		documentSelector: [
			{scheme: 'file', language: 'pine'},
			{scheme: 'untitled', language: 'pine'}
		],
	};

	const client = new LanguageClient(
		'yourExtension',
		'Your Extension Name',
		serverOptions,
		clientOptions
	);

	const disposable = client.start();
	context.subscriptions.push(disposable);
}


module.exports = {
	activate
};
