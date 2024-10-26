const onImageUpload = async function onImageUpload(e) {
	e.preventDefault();
	let formData = new FormData();
	let image = document.getElementById('image').files[0];

	if (!image) {
		alert('No image selected!');
	}

	if (image.size > 2 * 1024 * 1024) {
		alert('Max file size is 2MB');
		return;
	}

	formData.append('file', image);

	if (browser) {
		let xsrf_token = getCookie('xsrf_token');
		const response = await api.upload({
			method: 'POST',
			path: 'image-upload',
			data: formData,
			xsrf_token,
			headers: null
		});

		let text = await response.text();
		let j = text ? JSON.parse(text) : {};

		if (response.status === 200 && j.url) {
			closeForm();
			addImageURL(`![alt](${j.url})`);
		} else {
			alert(j.message);
		}
	}
};

export default onImageUpload;
