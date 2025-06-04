export class ApiError extends Error {
    constructor(public status: number, message: string) {
        super(message);
        this.name = 'ApiError';
    }
}

export async function handleApiResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
        let errorMessage = 'An error occurred';
        try {
            const errorData = await response.json();
            errorMessage = errorData.message || errorMessage;
        } catch (e) {
            errorMessage = response.statusText || errorMessage;
        }
        throw new ApiError(response.status, errorMessage);
    }

    try {
        // Check if the response has content
        const contentType = response.headers.get('content-type');
        if (contentType && contentType.includes('application/json')) {
            const text = await response.text();
            if (!text) {
                throw new Error('Empty response');
            }
            return JSON.parse(text) as T;
        } else if (response.status === 204) {
            // No content
            return {} as T;
        } else {
            throw new Error('Invalid response type');
        }
    } catch (error) {
        console.error('Error parsing response:', error);
        throw new ApiError(500, 'Failed to parse server response');
    }
}

export async function apiRequest<T>(
    url: string,
    options: RequestInit = {}
): Promise<T> {
    try {
        const response = await fetch(url, {
            ...options,
            headers: {
                'Content-Type': 'application/json',
                ...options.headers,
            },
        });
        return await handleApiResponse<T>(response);
    } catch (error) {
        if (error instanceof ApiError) {
            // Let the caller handle displaying the error
            throw error;
        }
        console.error('API request failed:', error);
        // Handle network/connection errors
        throw new ApiError(500, 'Failed to connect to server');
    }
}