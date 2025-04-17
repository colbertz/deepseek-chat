'use client'

import { useState } from 'react';
import { 
  MessageSquarePlus, 
  Download, 
  Settings,
  ChevronLeft
} from 'lucide-react';
import { useConversationFetcher } from './ConversationFetcher';
import { API_BASE_URL } from '../config/api';

interface NavbarProps {
  darkMode: boolean;
  setNavOpen: (open: boolean) => void;
  setMessages: (messages: Array<{content: string, role: string}>) => void;
  resetConversation: () => void;
  selectedConversationId: string | null;
  setSelectedConversationId: (id: string | null) => void;
}

export default function Navbar({ 
  darkMode, 
  setNavOpen, 
  setMessages,
  resetConversation,
  selectedConversationId,
  setSelectedConversationId
}: NavbarProps) {
  const { conversations, loading } = useConversationFetcher();

  const handleConversationClick = async (id: string) => {
    setSelectedConversationId(id);
    try {
      const response = await fetch(`${API_BASE_URL}/conversations/${id}`);
      const messages = await response.json();
      setMessages(messages);
    } catch (error) {
      console.error('Failed to fetch conversation:', error);
    }
  };
  
  return (
    <div 
      className={`w-64 
      ${darkMode ? 'bg-gray-800' : 'bg-gray-200'} 
      ${darkMode ? 'text-white' : 'text-gray-800'} 
      p-4 flex flex-col fixed h-full z-10`}
    >
      {/* Sidebar Header */}
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-2xl font-bold">DeepSeek</h1>
          <p className="text-gray-400 text-sm">AI Assistant</p>
        </div>
        <button 
          onClick={() => setNavOpen(false)}
          className={`p-1 rounded-full hover:${darkMode ? 'bg-gray-700' : 'bg-gray-300'}
            ${darkMode ? 'text-white' : 'text-gray-800'}`}
        >
          <ChevronLeft size={20} />
        </button>
      </div>

      {/* New Chat Button */}
      <button 
        className="flex items-center gap-2 bg-blue-600 hover:bg-blue-700 text-white py-2 px-4 rounded mb-6"
        onClick={() => {
          resetConversation();
          setSelectedConversationId(null);
        }}
      >
        <MessageSquarePlus size={18} />
        New Chat
      </button>

      {/* Chat History */}
      <div className="flex-1 overflow-y-auto pb-4">
        {loading ? (
          <div className="text-center py-4 text-gray-400">Loading...</div>
        ) : (
          <div className="space-y-4">
            {conversations.today.length > 0 && (
              <div>
                <h3 className="text-xs text-gray-400 mb-1">Today</h3>
                {conversations.today.map(conv => (
                  <div 
                    key={conv.id} 
                    onClick={() => handleConversationClick(conv.id)}
                    className={`group p-2 rounded cursor-pointer transition-all duration-200
                    ${darkMode ? 'hover:bg-gray-700' : 'hover:bg-gray-300'}
                    ${selectedConversationId === conv.id ? (darkMode ? 'bg-gray-700' : 'bg-gray-300') : ''}`}>
                    <span className={`block group-hover:scale-[1.02] transition-transform duration-200
                    ${darkMode ? 'group-hover:text-gray-100' : 'group-hover:text-gray-900'}`}>
                      {conv.title}
                    </span>
                  </div>
                ))}
              </div>
            )}
            {conversations.last7Days.length > 0 && (
              <div>
                <h3 className="text-xs text-gray-400 mb-1">Last 7 Days</h3>
                {conversations.last7Days.map(conv => (
                  <div 
                    key={conv.id} 
                    onClick={() => handleConversationClick(conv.id)}
                    className={`group p-2 rounded cursor-pointer transition-all duration-200
                    ${darkMode ? 'hover:bg-gray-700' : 'hover:bg-gray-300'}
                    ${selectedConversationId === conv.id ? (darkMode ? 'bg-gray-700' : 'bg-gray-300') : ''}`}>
                    <span className={`block group-hover:scale-[1.02] transition-transform duration-200
                    ${darkMode ? 'group-hover:text-gray-100' : 'group-hover:text-gray-900'}`}>
                      {conv.title}
                    </span>
                  </div>
                ))}
              </div>
            )}
            {conversations.last30Days.length > 0 && (
              <div>
                <h3 className="text-xs text-gray-400 mb-1">Last 30 Days</h3>
                {conversations.last30Days.map(conv => (
                  <div 
                    key={conv.id} 
                    onClick={() => handleConversationClick(conv.id)}
                    className={`group p-2 rounded cursor-pointer transition-all duration-200
                    ${darkMode ? 'hover:bg-gray-700' : 'hover:bg-gray-300'}
                    ${selectedConversationId === conv.id ? (darkMode ? 'bg-gray-700' : 'bg-gray-300') : ''}`}>
                    <span className={`block group-hover:scale-[1.02] transition-transform duration-200
                    ${darkMode ? 'group-hover:text-gray-100' : 'group-hover:text-gray-900'}`}>
                      {conv.title}
                    </span>
                  </div>
                ))}
              </div>
            )}
            {conversations.older.length > 0 && (
              <div>
                <h3 className="text-xs text-gray-400 mb-1">Older</h3>
                {conversations.older.map(conv => (
                  <div 
                    key={conv.id} 
                    onClick={() => handleConversationClick(conv.id)}
                    className={`group p-2 rounded cursor-pointer transition-all duration-200
                    ${darkMode ? 'hover:bg-gray-700' : 'hover:bg-gray-300'}
                    ${selectedConversationId === conv.id ? (darkMode ? 'bg-gray-700' : 'bg-gray-300') : ''}`}>
                    <span className={`block group-hover:scale-[1.02] transition-transform duration-200
                    ${darkMode ? 'group-hover:text-gray-100' : 'group-hover:text-gray-900'}`}>
                      {conv.title}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Bottom Section */}
      <div className="mt-auto">
        <button className={`flex items-center gap-2 py-2 px-4 rounded w-full
          ${darkMode ? 'text-gray-400 hover:text-white' : 'text-gray-500 hover:text-gray-800'}`}>
          <Download size={18} />
          Download App
        </button>
        <button className={`flex items-center gap-2 py-2 px-4 rounded w-full
          ${darkMode ? 'text-gray-400 hover:text-white' : 'text-gray-500 hover:text-gray-800'}`}>
          <Settings size={18} />
          Settings
        </button>
      </div>
    </div>
  );
}
