import { useEffect, useState } from 'react';
import { API_BASE_URL } from '../config/api';

interface Conversation {
  id: string;
  title: string;
  time: number;
}

interface GroupedConversations {
  today: Conversation[];
  last7Days: Conversation[];
  last30Days: Conversation[];
  older: Conversation[];
}

export const useConversationFetcher = () => {
  const [conversations, setConversations] = useState<GroupedConversations>({
    today: [],
    last7Days: [],
    last30Days: [],
    older: []
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchConversations = async () => {
      try {
        const response = await fetch(`${API_BASE_URL}/conversations`);
        const data: Conversation[] = await response.json();
        
        const now = new Date();
        const todayStart = new Date(now);
        todayStart.setHours(0, 0, 0, 0);
        
        const grouped = data.reduce((acc, conv) => {
          const convDate = new Date(conv.time);
          const diffDays = Math.floor((now.getTime() - convDate.getTime()) / (1000 * 60 * 60 * 24));
          
          if (convDate >= todayStart) {
            acc.today.push(conv);
          } else if (diffDays < 7) {
            acc.last7Days.push(conv);
          } else if (diffDays < 30) {
            acc.last30Days.push(conv);
          } else {
            acc.older.push(conv);
          }
          return acc;
        }, { today: [], last7Days: [], last30Days: [], older: [] } as GroupedConversations);

        setConversations(grouped);
      } catch (error) {
        console.error('Error fetching conversations:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchConversations();
  }, []);

  return { conversations, loading };
};
