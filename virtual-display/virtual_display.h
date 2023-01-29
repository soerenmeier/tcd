extern "C" {
	// returns true if this kind should be logged
	bool VdShouldLog(const char* kind);

	// logs a string if a logger is enabled
	void VdLog(const char* str);

	// returns a pointer to the data
	// this pointer should only be passed to rust
	// and never read on the c side
	// it is safe to pass it between threads
	const void* VdInitData();

	// Your only allowed to call this function once
	// and afterwards the pointer will be dangling.
	void VdDestroyData(const void* data);

	bool VdShouldSendTexture(const void* data, bool has_changed);

	// returns an array with exacly the len
	// you are not allowed to call this concurrently
	BYTE* VdCreateTextureBuffer(const void* data, DWORD len);

	// you are not allowed to call this concurrently
	void VdSendTexture(const void* data, BYTE* ptr, DWORD len);
}