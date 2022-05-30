
/// returns a url object with the api + added path
export function getUrl(path) {
	const host = window.location.host;
	const url = new URL('http://' + host + '/api' + path);
	if (IN_DEBUG)
		url.port = 3511;
	return url;
}