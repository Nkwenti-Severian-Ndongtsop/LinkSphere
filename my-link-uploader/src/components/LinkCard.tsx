import React, { useState } from 'react';
import { LinkIcon, ExternalLink, User as UserIcon, Calendar, Clock, Pencil } from 'lucide-react';
import { formatInTimeZone } from 'date-fns-tz';
import ConfirmationModal from './ConfirmationModal';
import SuccessModal from './SuccessModal';
import { useNavigate } from 'react-router-dom';

interface LinkPreview {
  title?: string;
  description?: string;
  image?: string;
  favicon?: string;
}

interface Link {
  id: string;
  url: string;
  title: string;
  description: string;
  click_count: number;
  created_at: string;
  user_id: string;
  preview?: LinkPreview;
  user?: { username: string };
}

interface User {
  id: string;
  username: string;
}

interface LinkCardProps {
  link: Link;
  currentUser: User | null;
  onDelete?: (id: string) => void;
}

const LinkCard: React.FC<LinkCardProps> = ({ link, currentUser, onDelete }) => {
  const [imageError, setImageError] = useState(false);
  const [faviconError, setFaviconError] = useState(false);
  const [showModal, setShowModal] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const navigate = useNavigate();

  const isOwner = currentUser && link.user_id === currentUser.id;
  const hasImage = !!link.preview?.image && !imageError;

  // Helper: is link editable (within 1 minute for testing)?
  const isEditable = () => {
    const created = new Date(link.created_at);
    const now = new Date();
    return (now.getTime() - created.getTime()) < 60 * 1000; // 1 minute
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowModal(true);
  };

  const handleConfirmDelete = () => {
    if (onDelete) onDelete(link.id);
    setShowModal(false);
    setShowSuccess(true);
    setTimeout(() => setShowSuccess(false), 2000);
  };

  const handleCloseModal = () => {
    setShowModal(false);
  };

  const handleCloseSuccess = () => {
    setShowSuccess(false);
  };

  const handleVisit = (e: React.MouseEvent) => {
    e.stopPropagation();
    window.open(link.url, '_blank', 'noopener,noreferrer');
  };

  const handleEdit = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowEditModal(true);
  };
  const handleConfirmEdit = () => {
    setShowEditModal(false);
    navigate(`/dashboard/upload/${link.id}/edit`);
  };
  const handleCloseEditModal = () => {
    setShowEditModal(false);
  };

  const formatDate = (dateString: string) => {
    try {
      const date = new Date(dateString);
      return formatInTimeZone(date, 'Africa/Douala', 'MMM d, yyyy');
    } catch (error) {
      return 'Invalid date';
    }
  };

  const formatTime = (dateString: string) => {
    try {
      const date = new Date(dateString);
      return formatInTimeZone(date, 'Africa/Douala', 'HH:mm');
    } catch (error) {
      return 'Invalid time';
    }
  };

  // Layout for owned links with images
  if (isOwner && hasImage) {
    return (
      <div className="bg-gray-800 rounded-lg shadow-lg overflow-hidden flex flex-col">
        <div className="relative aspect-video">
          <img
            src={link.preview!.image}
            alt={link.title}
            className="w-full h-full object-cover"
            onError={() => setImageError(true)}
          />
          {link.preview?.favicon && !faviconError && (
            <img
              src={link.preview.favicon}
              alt="favicon"
              className="absolute top-2 left-2 w-6 h-6 rounded-full bg-white"
              onError={() => setFaviconError(true)}
            />
          )}
        </div>
        <div className="p-4 flex-1 flex flex-col">
          <h3 className="text-xl font-semibold mb-1 text-gray-100">{link.title}</h3>
          <p className="mb-2 text-gray-400 line-clamp-2">{link.description}</p>
          <div className="flex items-center gap-4 mb-2 text-sm text-gray-400">
            <div className="flex items-center gap-1">
              <UserIcon size={16} />
              <span>{link.user?.username || 'Unknown user'}</span>
            </div>
            <div className="flex items-center gap-1">
              <Calendar size={16} />
              <span>{formatDate(link.created_at)}</span>
            </div>
            <div className="flex items-center gap-1">
              <Clock size={16} />
              <span>{formatTime(link.created_at)}</span>
            </div>
          </div>
          <div className="mb-4 text-sm text-gray-400">
            {link.click_count} clicks
          </div>
          <div className="mt-auto flex flex-col gap-2">
            <button
              className="w-full flex items-center justify-center space-x-2 px-4 py-3 rounded-lg transition-all duration-300 bg-[#231942] text-purple-400 hover:bg-[#2d2350]"
              onClick={handleVisit}
            >
              <ExternalLink size={20} />
              <span>Visit Link</span>
            </button>
            {isEditable() ? (
              <div className="flex gap-2">
                <button
                  className="flex-1 flex items-center justify-center space-x-2 px-4 py-2 rounded-xl transition-all duration-300 bg-orange-400 text-white hover:bg-orange-500 text-sm font-semibold"
                  onClick={handleEdit}
                >
                  Edit
                </button>
                <button
                  className="flex-1 flex items-center justify-center space-x-2 px-4 py-2 rounded-xl transition-all duration-300 bg-red-600 text-white hover:bg-red-700 text-sm font-semibold"
                  onClick={handleDelete}
                >
                  Delete
                </button>
              </div>
            ) : (
              <button
                className="w-full px-3 py-2 rounded-xl bg-red-600 text-white hover:bg-red-700 transition text-sm font-semibold"
                onClick={handleDelete}
              >
                Delete
              </button>
            )}
            <ConfirmationModal
              isOpen={showModal}
              onClose={handleCloseModal}
              onConfirm={handleConfirmDelete}
              message="Are you sure you want to delete this link?"
            />
            <SuccessModal
              isOpen={showSuccess}
              onClose={handleCloseSuccess}
              message="Link deleted successfully!"
            />
            <EditConfirmationModal
              isOpen={showEditModal}
              onClose={handleCloseEditModal}
              onConfirm={handleConfirmEdit}
              message="Are you sure you want to edit this link?"
            />
          </div>
        </div>
      </div>
    );
  }

  // Layout for owned links without images
  if (isOwner && !hasImage) {
    return (
      <div className="bg-gray-800 rounded-lg shadow-lg overflow-hidden flex flex-col">
        <div className="flex-1 flex flex-col p-4">
          <div className="w-full h-32 flex items-center justify-center bg-gradient-to-br from-purple-600 to-pink-500 rounded mb-4">
      <LinkIcon size={48} className="text-white" />
          </div>
          <h3 className="text-xl font-semibold mb-1 text-gray-100">{link.title}</h3>
          <p className="mb-2 text-gray-400 line-clamp-2">{link.description}</p>
          <div className="flex items-center gap-4 mb-2 text-sm text-gray-400">
            <div className="flex items-center gap-1">
              <UserIcon size={16} />
              <span>{link.user?.username || 'Unknown user'}</span>
            </div>
            <div className="flex items-center gap-1">
              <Calendar size={16} />
              <span>{formatDate(link.created_at)}</span>
            </div>
            <div className="flex items-center gap-1">
              <Clock size={16} />
              <span>{formatTime(link.created_at)}</span>
            </div>
          </div>
          <div className="mb-4 text-sm text-gray-400">
            {link.click_count} clicks
          </div>
          <button
            className="w-full flex items-center justify-center space-x-2 px-4 py-3 rounded-lg transition-all duration-300 bg-[#231942] text-purple-400 hover:bg-[#2d2350]"
            onClick={handleVisit}
          >
            <ExternalLink size={20} />
            <span>Visit Link</span>
          </button>
          {isEditable() ? (
            <div className="flex gap-2 mt-2">
              <button
                className="flex-1 flex items-center justify-center space-x-2 px-4 py-2 rounded-xl transition-all duration-300 bg-orange-400 text-white hover:bg-orange-500 text-sm font-semibold"
                onClick={handleEdit}
              >
                Edit
              </button>
              <button
                className="flex-1 flex items-center justify-center space-x-2 px-4 py-2 rounded-xl transition-all duration-300 bg-red-600 text-white hover:bg-red-700 text-sm font-semibold"
                onClick={handleDelete}
              >
                Delete
              </button>
            </div>
          ) : (
            <button
              className="w-full mt-2 px-3 py-2 rounded-xl bg-red-600 text-white hover:bg-red-700 transition text-sm font-semibold"
              onClick={handleDelete}
            >
              Delete
            </button>
          )}
          <ConfirmationModal
            isOpen={showModal}
            onClose={handleCloseModal}
            onConfirm={handleConfirmDelete}
            message="Are you sure you want to delete this link?"
          />
          <SuccessModal
            isOpen={showSuccess}
            onClose={handleCloseSuccess}
            message="Link deleted successfully!"
          />
          <EditConfirmationModal
            isOpen={showEditModal}
            onClose={handleCloseEditModal}
            onConfirm={handleConfirmEdit}
            message="Are you sure you want to edit this link?"
          />
        </div>
    </div>
  );
  }

  // Layout for links not owned by the user
  return (
    <div className="bg-gray-800 rounded-lg shadow-lg overflow-hidden flex flex-col">
      <div className="relative aspect-video">
        {hasImage ? (
          <img
            src={link.preview!.image}
            alt={link.title}
            className="w-full h-full object-cover"
            onError={() => setImageError(true)}
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-purple-600 to-pink-500">
            <LinkIcon size={48} className="text-white" />
          </div>
        )}
        {link.preview?.favicon && !faviconError && (
          <img
            src={link.preview.favicon}
            alt="favicon"
            className="absolute top-2 left-2 w-6 h-6 rounded-full bg-white"
            onError={() => setFaviconError(true)}
          />
        )}
      </div>
      <div className="p-4 flex-1 flex flex-col">
        <h3 className="text-xl font-semibold mb-1 text-gray-100">{link.title}</h3>
        <p className="mb-2 text-gray-400 line-clamp-2">{link.description}</p>
        <div className="flex items-center gap-4 mb-2 text-sm text-gray-400">
          <div className="flex items-center gap-1">
            <UserIcon size={16} />
            <span>{link.user?.username || 'Unknown user'}</span>
          </div>
          <div className="flex items-center gap-1">
            <Calendar size={16} />
          <span>{formatDate(link.created_at)}</span>
          </div>
          <div className="flex items-center gap-1">
            <Clock size={16} />
            <span>{formatTime(link.created_at)}</span>
          </div>
        </div>
        <div className="mb-4 text-sm text-gray-400">
          {link.click_count} clicks
        </div>
        <button
          className="w-full flex items-center justify-center space-x-2 px-4 py-3 rounded-lg transition-all duration-300 bg-[#231942] text-purple-400 hover:bg-[#2d2350]"
          onClick={handleVisit}
        >
          <ExternalLink size={20} />
          <span>Visit Link</span>
        </button>
      </div>
    </div>
  );
};

function EditConfirmationModal({ isOpen, onClose, onConfirm, message }: { isOpen: boolean; onClose: () => void; onConfirm: () => void; message: string }) {
  if (!isOpen) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black bg-opacity-60 transition-opacity" onClick={onClose} />
      <div className="relative z-10 bg-gray-900 rounded-xl shadow-2xl p-8 w-full max-w-md mx-auto flex flex-col items-center border border-gray-700">
        <div className="flex flex-col items-center mb-6">
          <div className="w-16 h-16 flex items-center justify-center rounded-full bg-gradient-to-br from-orange-400 to-yellow-500 mb-4">
            <Pencil size={36} className="text-white" />
          </div>
          <p className="text-lg text-gray-100 font-semibold text-center">{message}</p>
        </div>
        <div className="flex w-full justify-center gap-4 mt-4">
          <button className="px-5 py-2 rounded-lg bg-gray-700 text-gray-200 hover:bg-gray-600 transition font-medium" onClick={onClose}>Cancel</button>
          <button className="px-5 py-2 rounded-lg bg-gradient-to-r from-orange-400 to-yellow-500 text-white hover:from-orange-500 hover:to-yellow-400 transition font-semibold shadow" onClick={onConfirm}>Yes, Edit</button>
        </div>
      </div>
    </div>
  );
}

export default LinkCard; 