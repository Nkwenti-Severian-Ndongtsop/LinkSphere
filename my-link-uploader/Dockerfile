# Production stage: only serve pre-built static files
FROM nginx:alpine

# Copy built assets from the host (assumes you have run the build already)
COPY frontend/package/* /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# Start Nginx
CMD ["nginx", "-g", "daemon off;"] 