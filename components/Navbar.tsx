'use client'

import { 
  MessageSquarePlus, 
  Download, 
  Settings,
  ChevronLeft
} from 'lucide-react';

interface NavbarProps {
  darkMode: boolean;
  setNavOpen: (open: boolean) => void;
}

export default function Navbar({ darkMode, setNavOpen }: NavbarProps) {
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
      <button className="flex items-center gap-2 bg-blue-600 hover:bg-blue-700 text-white py-2 px-4 rounded mb-6">
        <MessageSquarePlus size={18} />
        New Chat
      </button>

      {/* Chat History */}
      <div className="flex-1 overflow-y-auto">
        <h2 className="text-gray-400 text-sm mb-2">RECENT CHATS</h2>
        <div className="space-y-1">
          {[1, 2, 3].map((item) => (
            <div key={item} className={`p-2 rounded cursor-pointer 
            hover:${darkMode ? 'bg-gray-700' : 'bg-gray-300'}`}>
              Chat {item}
            </div>
          ))}
        </div>
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
