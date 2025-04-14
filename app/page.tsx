'use client'

import { useState } from 'react';
import Navbar from '../components/Navbar';
import Conversation from '../components/Conversation';

export default function Home() {
  const [darkMode, setDarkMode] = useState(true);
  const [navOpen, setNavOpen] = useState(true);

  return (
    <div className={`flex h-screen ${darkMode ? 'bg-gray-900' : 'bg-gray-100'}`}>
      {/* Left Sidebar */}
      {navOpen && <Navbar darkMode={darkMode} setNavOpen={setNavOpen} />}

      {/* Main Content Area */}
      <Conversation 
        darkMode={darkMode}
        navOpen={navOpen}
        setDarkMode={setDarkMode}
        setNavOpen={setNavOpen}
      />
    </div>
  );
}
