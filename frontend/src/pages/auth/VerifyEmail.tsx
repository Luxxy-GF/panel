import { faExclamationTriangle } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { useEffect, useRef, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router';
import verifyEmail from '@/api/auth/verifyEmail.ts';
import { httpErrorToHuman } from '@/api/axios.ts';
import Alert from '@/elements/Alert.tsx';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import Spinner from '@/elements/Spinner.tsx';
import Stack from '@/elements/Stack.tsx';
import Text from '@/elements/Text.tsx';
import Title from '@/elements/Title.tsx';
import { useAuth } from '@/providers/AuthProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import AuthWrapper from './AuthWrapper.tsx';

export default function VerifyEmail() {
  const { doLogin } = useAuth();
  const { t } = useTranslations();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const token = searchParams.get('token');

  const [error, setError] = useState('');
  const attempted = useRef(false);

  useEffect(() => {
    if (attempted.current) {
      return;
    }
    attempted.current = true;

    if (!token) {
      navigate('/auth/login');
      return;
    }

    verifyEmail(token)
      .then((response) => {
        doLogin(response.user);
      })
      .catch((msg) => {
        setError(httpErrorToHuman(msg));
      });
  }, []);

  return (
    <AuthWrapper>
      <div className='flex flex-col space-y-4 mb-4 w-full'>
        {error && (
          <Alert
            icon={<FontAwesomeIcon icon={faExclamationTriangle} />}
            color='red'
            title={t('common.alert.error', {})}
          >
            {error}
          </Alert>
        )}
      </div>

      <Stack className='w-full'>
        <div>
          <Title order={2}>{t('pages.auth.verifyEmail.title', {})}</Title>
          <Text className='text-neutral-400!'>{t('pages.auth.verifyEmail.subtitle', {})}</Text>
        </div>
        <Card>
          <Stack>
            {error ? (
              <Button variant='light' onClick={() => navigate('/auth/login')} size='md' fullWidth>
                {t('pages.auth.button.login', {})}
              </Button>
            ) : (
              <div className='flex items-center justify-center py-4'>
                <Spinner />
              </div>
            )}
          </Stack>
        </Card>
      </Stack>
    </AuthWrapper>
  );
}
