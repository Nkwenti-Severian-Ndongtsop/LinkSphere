# Changelog

All notable changes to the LinkSphere project will be documented in this file.

## [Unreleased]

### Added
- Initial project setup with Rust backend and React TypeScript frontend
- Complete database schema implementation
  - Users table with authentication fields
  - Links table for bookmark storage
  - Categories and Tags for organization
  - Junction tables for many-to-many relationships
  - Automatic timestamp management

#### Backend Features
- User authentication system
  - JWT-based authentication
  - Password hashing with Argon2
  - Email verification system
  - Password reset functionality
- Rate limiting middleware
- CORS configuration for frontend integration
- RESTful API endpoints:
  - `/api/auth/login`
  - `/api/auth/register`
  - `/api/auth/logout`
  - `/api/users`
  - `/api/links` (GET, POST)
  - `/api/links/:id` (DELETE)
  - `/api/links/:id/click`
- Database migrations system
- Protected routes middleware
- Error handling system

#### Frontend Features
- React application with TypeScript
- Routing system with protected routes
- Authentication flow
- Theme provider for dark/light mode
- Components:
  - Layout component
  - HomePage
  - UploadForm
  - AdminDashboard
  - Login/Signup pages
- Animation support with Framer Motion
- Vite build configuration
- TypeScript configuration
- Path aliases for cleaner imports

### Database Schema
- Users
  ```sql
  - id (UUID)
  - username
  - email
  - password_hash
  - user_role (enum: user, admin)
  - is_email_verified
  - verification_token
  - verification_token_expires_at
  - reset_token
  - reset_token_expires_at
  - created_at
  - updated_at
  ```
- Links
  ```sql
  - id (SERIAL)
  - user_id (UUID)
  - url
  - title
  - description
  - click_count
  - favicon_url
  - created_at
  - updated_at
  ```
- Categories
  ```sql
  - id (SERIAL)
  - name
  - description
  - user_id (UUID)
  - created_at
  - updated_at
  ```
- Tags
  ```sql
  - id (SERIAL)
  - name
  - user_id (UUID)
  - created_at
  - updated_at
  ```

### Technical Details
- Backend:
  - Rust with Axum web framework
  - PostgreSQL database with SQLx
  - JWT authentication
  - Argon2 password hashing
  - Async/await support
  - Error handling with custom types
  - Database connection pooling

- Frontend:
  - React 18 with TypeScript
  - Vite build tool
  - React Router v6
  - Framer Motion for animations
  - Context API for state management
  - Protected route system
  - Proxy configuration for API requests

### Development Setup
- Environment configuration
- Database migration system
- Development server configuration
- TypeScript strict mode
- Path aliases
- Build optimization settings

### Security Features
- JWT-based authentication
- Password hashing with Argon2
- Rate limiting
- CORS protection
- Protected routes
- Secure password reset flow
- Email verification system

### Next Steps
- Implement social features
- Add WebSocket support for real-time updates
- Enhance error handling
- Add comprehensive testing
- Implement caching system
- Add analytics dashboard
- Implement search functionality
- Add tag and category management UI

## [0.1.0] - Initial Release
- Basic project structure
- Core features implementation
- Database schema setup
- Authentication system
- Frontend setup with React 