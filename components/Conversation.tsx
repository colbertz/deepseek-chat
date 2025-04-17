'use client'

import { Sun, Moon, Menu } from 'lucide-react';

interface ConversationProps {
  darkMode: boolean;
  navOpen: boolean;
  setDarkMode: (mode: boolean) => void;
  setNavOpen: (open: boolean) => void;
  messages: Array<{content: string, role: string}>;
}

export default function Conversation({
  darkMode,
  navOpen,
  setDarkMode,
  setNavOpen,
  messages
}: ConversationProps) {
  return (
    <div 
      className={`flex-1 flex flex-col ${navOpen ? 'ml-64' : 'ml-0'} 
      ${darkMode ? 'bg-gray-900' : 'bg-white'} 
      transition-all duration-300`}
    >
      {/* Menu Button when sidebar is closed */}
      {!navOpen && (
        <button 
          onClick={() => setNavOpen(true)}
          className={`absolute top-4 left-4 p-1 rounded-full z-10
            hover:${darkMode ? 'bg-gray-700' : 'bg-gray-300'}
            ${darkMode ? 'text-white' : 'text-gray-800'}`}
        >
          <Menu size={20} />
        </button>
      )}

      {/* Theme Toggle Button */}
      <button
        onClick={() => setDarkMode(!darkMode)}
        className={`absolute top-4 right-4 p-1 rounded-full z-10
          hover:${darkMode ? 'bg-gray-700' : 'bg-gray-300'}
          ${darkMode ? 'text-white' : 'text-gray-800'}`}
      >
        {darkMode ? <Sun size={20} /> : <Moon size={20} />}
      </button>

      {/* Message Area */}
      <div className="flex-1 p-6 overflow-y-auto">
        {messages.length > 0 ? (
          <div className="max-w-3xl mx-auto space-y-4">
            {messages.map((message: {content: string, role: string}, index: number) => (
              <div 
                key={index}
                className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
              >
                <div 
                  className={`max-w-[80%] rounded-lg px-4 py-2 ${
                    message.role === 'user' 
                      ? darkMode 
                        ? 'bg-blue-600 text-white' 
                        : 'bg-blue-500 text-white'
                      : darkMode 
                        ? 'bg-gray-700 text-white' 
                        : 'bg-gray-200 text-gray-800'
                  }`}
                >
                  {message.content}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="max-w-3xl mx-auto text-center">
              <h1 className={`text-4xl font-bold mb-4 ${darkMode ? 'text-white' : 'text-gray-900'}`}>
                Hello, I'm DeepSeek
              </h1>
              <p className={`text-lg ${darkMode ? 'text-gray-300' : 'text-gray-600'}`}>
                How can I help you today?
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Input Area */}
      <div className={`p-4 border-t ${darkMode ? 'border-gray-700' : 'border-gray-200'}`}>
        <div className="max-w-3xl mx-auto flex">
          <input
            type="text"
            placeholder="Type your message here..."
            className={`flex-1 border rounded-l-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 
            ${darkMode ? 'border-gray-600 bg-gray-800 text-white' : 'border-gray-300 bg-gray-100 text-gray-900'}`}
          />
          <button className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-r-lg">
            Send
          </button>
        </div>
      </div>
    </div>
  );
}
